use std::path::Path;
use std::sync::Arc;

use crate::{
    core::worktree_context::WorktreeContextService,
    db::Db,
    dispatch::{
        seams::{launch_agent, AgentLauncher, PrPublisher},
        DispatchEvent, DispatchHandle,
    },
    error::AppError,
    git::{
        worktree_paths::{branch_name, worktree_path},
        GitService,
    },
    models::{ChangedFile, Task, TaskStatus, VerificationStatus},
    process::ProcessRunner,
    projects::ProjectService,
    tasks::TaskService,
    verification::VerificationService,
};
use tauri::AppHandle;
use tauri_specta::Event;

enum AgentOutcome {
    Success,
    Errored,
    Stopped,
}

pub struct DispatchWorker {
    pub tasks: TaskService,
    pub projects: ProjectService,
    pub verification: VerificationService,
    pub git: GitService,
    pub worktree_context: WorktreeContextService,
    pub process: Arc<ProcessRunner>,
    pub agent: Arc<dyn AgentLauncher>,
    pub publisher: Arc<dyn PrPublisher>,
    pub handle: DispatchHandle,
    pub app: Option<AppHandle>,
}

impl DispatchWorker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Db,
        process: Arc<ProcessRunner>,
        agent: Arc<dyn AgentLauncher>,
        publisher: Arc<dyn PrPublisher>,
        handle: DispatchHandle,
        app: Option<AppHandle>,
    ) -> Self {
        Self {
            tasks: TaskService::new(db.clone()),
            projects: ProjectService::new(db.clone()),
            verification: VerificationService::new(db.clone()),
            git: GitService::new(),
            worktree_context: WorktreeContextService::new(),
            process,
            agent,
            publisher,
            handle,
            app,
        }
    }

    /// Long-lived loop. Pulls one queued task at a time, drives it to a terminal
    /// state, then pulls the next. Sleeps on `notify` when idle or paused.
    pub async fn run(self: Arc<Self>) {
        loop {
            if self.handle.is_paused() {
                self.handle.notify.notified().await;
                continue;
            }
            match self.tasks.next_queued().await {
                Ok(Some(task)) => {
                    *self.handle.current_task.lock().await = Some(task.id.clone());
                    self.run_pipeline(&task).await;
                    *self.handle.current_task.lock().await = None;
                }
                Ok(None) => self.handle.notify.notified().await,
                Err(_) => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
            }
        }
    }

    fn emit(&self, task_id: &str, stage: &str, outcome: &str) {
        if let Some(app) = &self.app {
            let _ = DispatchEvent {
                task_id: task_id.to_string(),
                stage: stage.to_string(),
                outcome: outcome.to_string(),
            }
            .emit(app);
        }
    }

    async fn set(&self, task_id: &str, status: TaskStatus) {
        let _ = self.tasks.update_status(task_id, status).await;
    }

    async fn git_status(&self, worktree: &Path) -> Result<Vec<ChangedFile>, AppError> {
        let p = worktree.to_path_buf();
        tokio::task::spawn_blocking(move || crate::git::status::status(&p))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))?
    }

    /// Drive one task from Queued to a terminal state. Never panics; all errors
    /// land the task in a resting status and the loop continues.
    /// `pub(crate)` so `worker_tests` (a sibling module) can drive it directly.
    pub(crate) async fn run_pipeline(&self, task: &Task) {
        // Codex guard (defensive — enqueue already rejects non-claude).
        let engine = task.selected_engine.as_deref().unwrap_or("claude-code");
        if engine != "claude-code" {
            self.emit(&task.id, "guard", "unsupported-engine");
            self.set(&task.id, TaskStatus::Failed).await;
            return;
        }

        // Stage 1: worktree.
        self.emit(&task.id, "worktree", "start");
        let project = match self.projects.get(&task.project_id).await {
            Ok(p) => p,
            Err(_) => return self.fail(&task.id, "worktree").await,
        };
        let branch = branch_name(&task.id);
        let dest = match worktree_path(&task.project_id, &task.id) {
            Ok(d) => d,
            Err(_) => return self.fail(&task.id, "worktree").await,
        };
        let repo = std::path::PathBuf::from(&project.path);
        // Capture the returned Task — it now carries branch_name + worktree_path,
        // which the PR stage needs. The original `task` arg is still None for those.
        let task = match crate::core::worktree_lifecycle::create_worktree_lifecycle(
            &self.git,
            &self.worktree_context,
            &self.tasks,
            task,
            &repo,
            &branch,
            &dest,
            &project.default_branch,
        )
        .await
        {
            Ok(t) => t,
            Err(_) => return self.fail(&task.id, "worktree").await,
        };

        // Stage 2: agent (one retry).
        self.emit(&task.id, "agent", "start");
        match self.run_agent_with_retry(&task.id, &dest).await {
            AgentOutcome::Success => {}
            AgentOutcome::Stopped => {
                self.emit(&task.id, "agent", "stopped");
                return self.set(&task.id, TaskStatus::Stopped).await;
            }
            AgentOutcome::Errored => return self.fail(&task.id, "agent").await,
        }

        // Stage 3: reconcile.
        let changed = match self.git_status(&dest).await {
            Ok(c) => c,
            Err(_) => return self.fail(&task.id, "reconcile").await,
        };
        if changed.is_empty() {
            self.emit(&task.id, "reconcile", "no-changes");
            return self.set(&task.id, TaskStatus::Stopped).await;
        }

        // Stage 4: verification (one re-run). Failure stops at ReviewReady, no PR.
        self.emit(&task.id, "verify", "start");
        self.set(&task.id, TaskStatus::VerificationRunning).await;
        if !self.verify_with_retry(&task.project_id, &task.id, &dest).await {
            self.emit(&task.id, "verify", "failed");
            return self.set(&task.id, TaskStatus::ReviewReady).await;
        }

        // Stage 5: draft PR (one retry). Failure stops at ReviewReady (work is good).
        self.emit(&task.id, "pr", "start");
        if !self.publish_pr_with_retry(&task, &project.default_branch, &dest).await {
            self.emit(&task.id, "pr", "failed");
            return self.set(&task.id, TaskStatus::ReviewReady).await;
        }
        self.set(&task.id, TaskStatus::PrPrepared).await;
        self.emit(&task.id, "pr", "ok");
    }

    async fn fail(&self, task_id: &str, stage: &str) {
        self.emit(task_id, stage, "failed");
        self.set(task_id, TaskStatus::Failed).await;
    }

    async fn run_agent(&self, task_id: &str, worktree: &Path) -> AgentOutcome {
        self.set(task_id, TaskStatus::Running).await;
        if launch_agent(self.agent.as_ref(), &self.process, task_id, worktree)
            .await
            .is_err()
        {
            return AgentOutcome::Errored;
        }
        let info = self.process.wait_for_exit(task_id).await;
        if info.stopped_by_user {
            AgentOutcome::Stopped
        } else if info.exit_code == Some(0) && !info.signaled {
            AgentOutcome::Success
        } else {
            AgentOutcome::Errored
        }
    }

    async fn run_agent_with_retry(&self, task_id: &str, worktree: &Path) -> AgentOutcome {
        match self.run_agent(task_id, worktree).await {
            AgentOutcome::Errored => {
                self.emit(task_id, "agent", "retry");
                self.run_agent(task_id, worktree).await
            }
            other => other,
        }
    }

    async fn verify_once(&self, project_id: &str, task_id: &str, worktree: &Path) -> bool {
        match self.verification.run_for_task(project_id, task_id, worktree).await {
            Ok(run) => !run.checks.iter().any(|c| c.status == VerificationStatus::Failed),
            Err(_) => false,
        }
    }

    async fn verify_with_retry(&self, project_id: &str, task_id: &str, worktree: &Path) -> bool {
        if self.verify_once(project_id, task_id, worktree).await {
            return true;
        }
        self.emit(task_id, "verify", "retry");
        self.verify_once(project_id, task_id, worktree).await
    }

    async fn publish_once(&self, task: &Task, base: &str, worktree: &Path) -> Result<(), AppError> {
        let branch = task
            .branch_name
            .clone()
            .ok_or_else(|| AppError::invalid_arg("task has no branch"))?;
        let runs = self.verification.list_for_task(&task.id).await?;
        let latest = runs.first().cloned();
        let changed = self.git_status(worktree).await?;
        let body = crate::commands::pr::report::render(task, &changed, latest.as_ref());

        let publisher = self.publisher.clone();
        let repo = worktree.to_path_buf();
        let branch_c = branch.clone();
        tokio::task::spawn_blocking(move || publisher.push_branch(&repo, &branch_c))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))??;

        let publisher = self.publisher.clone();
        let repo = worktree.to_path_buf();
        let title = task.title.clone();
        let base_c = base.to_string();
        tokio::task::spawn_blocking(move || {
            publisher.create_pr(&repo, &title, &body, &base_c, true)
        })
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))??;
        Ok(())
    }

    async fn publish_pr_with_retry(&self, task: &Task, base: &str, worktree: &Path) -> bool {
        if self.publish_once(task, base, worktree).await.is_ok() {
            return true;
        }
        self.emit(&task.id, "pr", "retry");
        self.publish_once(task, base, worktree).await.is_ok()
    }
}
