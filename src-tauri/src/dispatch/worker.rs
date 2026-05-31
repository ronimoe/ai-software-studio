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

    // TEMP stub — real implementation lands in Task 7 (run_pipeline + retry).
    async fn run_pipeline(&self, _task: &Task) {}
}
