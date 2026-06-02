use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::{
    db::Db,
    dispatch::{
        seams::{AgentLauncher, PrPublisher},
        worker::DispatchWorker,
        DispatchHandle,
    },
    error::AppError,
    models::{CreateTaskRequest, TaskStatus, VerificationSettings},
    process::ProcessRunner,
};

// --- Fakes ---

/// Agent that runs `sh -c <script>` in the worktree.
struct FakeAgent {
    script: String,
}
impl AgentLauncher for FakeAgent {
    fn command(&self, _task_id: &str) -> Result<(String, Vec<String>), AppError> {
        Ok(("sh".to_string(), vec!["-c".to_string(), self.script.clone()]))
    }
}

#[derive(Default, Clone)]
struct FakePublisher {
    calls: Arc<Mutex<u32>>,
}
impl PrPublisher for FakePublisher {
    fn push_branch(&self, _repo: &Path, _branch: &str) -> Result<(), AppError> {
        Ok(())
    }
    fn create_pr(&self, _r: &Path, _t: &str, _b: &str, _base: &str, _draft: bool)
        -> Result<String, AppError> {
        *self.calls.lock().unwrap() += 1;
        Ok("https://github.com/x/y/pull/1".to_string())
    }
}

// --- Harness ---

fn run_git(dir: &Path, args: &[&str]) {
    std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
}

/// Owns everything a seeded test created on disk and reclaims it on drop:
/// the source repo `TempDir` (auto-removed) and the global worktree the pipeline
/// creates under `worktree_root()/<project_id>` (removed best-effort). Tests must
/// bind it for their whole body — dropping it early would delete the repo mid-test.
struct SeedGuard {
    _src: tempfile::TempDir,
    project_id: String,
}

impl Drop for SeedGuard {
    fn drop(&mut self) {
        if let Ok(root) = crate::git::worktree_paths::worktree_root() {
            let _ = std::fs::remove_dir_all(root.join(&self.project_id));
        }
    }
}

/// In-memory DB + a real git repo registered as a project + a queued claude-code task.
/// Returns (guard, project_id, task_id); keep `guard` alive for the test's duration.
async fn seed(db: &Db) -> (SeedGuard, String, String) {
    let src = tempfile::TempDir::new().unwrap();
    let dir = src.path().to_path_buf();
    run_git(&dir, &["init", "-q", "-b", "main"]);
    std::fs::write(dir.join("README.md"), "x").unwrap();
    run_git(&dir, &["add", "."]);
    run_git(&dir, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"]);

    let project_id = format!("proj-{}", uuid::Uuid::new_v4());
    sqlx::query("INSERT INTO projects (id, name, path, default_branch) VALUES (?, 'r', ?, 'main')")
        .bind(&project_id)
        .bind(dir.to_string_lossy().as_ref())
        .execute(&db.pool)
        .await
        .unwrap();

    let tasks = crate::tasks::TaskService::new(db.clone());
    let task = tasks
        .create(&CreateTaskRequest {
            project_id: project_id.clone(),
            title: "t".into(),
            description: "d".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec!["ac".into()],
            constraints: vec![],
            selected_engine: Some("claude-code".into()),
        })
        .await
        .unwrap();
    tasks.enqueue(&task.id).await.unwrap();
    (
        SeedGuard { _src: src, project_id: project_id.clone() },
        project_id,
        task.id,
    )
}

fn worker(db: &Db, agent: FakeAgent, publisher: FakePublisher) -> Arc<DispatchWorker> {
    Arc::new(DispatchWorker::new(
        db.clone(),
        Arc::new(ProcessRunner::new()),
        Arc::new(agent),
        Arc::new(publisher),
        DispatchHandle::new(false),
        None,
    ))
}

fn settings(test: &str) -> VerificationSettings {
    VerificationSettings {
        install: Some("true".into()),
        typecheck: Some("true".into()),
        lint: Some("true".into()),
        test: Some(test.into()),
        build: Some("true".into()),
    }
}

// --- Tests ---

#[tokio::test]
async fn pipeline_reaches_pr_prepared_when_changes_and_verification_passes() {
    let db = Db::test_pool().await.unwrap();
    let (_guard, project_id, task_id) = seed(&db).await;
    crate::verification::VerificationService::new(db.clone())
        .set_settings(&project_id, &settings("true"))
        .await
        .unwrap();

    let pub_fake = FakePublisher::default();
    let w = worker(&db, FakeAgent { script: "echo hi > new.txt".into() }, pub_fake.clone());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;

    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::PrPrepared);
    assert_eq!(*pub_fake.calls.lock().unwrap(), 1);
}

#[tokio::test]
async fn pipeline_stops_review_ready_when_verification_fails() {
    let db = Db::test_pool().await.unwrap();
    let (_guard, project_id, task_id) = seed(&db).await;
    crate::verification::VerificationService::new(db.clone())
        .set_settings(&project_id, &settings("false")) // failing test command
        .await
        .unwrap();

    let pub_fake = FakePublisher::default();
    let w = worker(&db, FakeAgent { script: "echo hi > new.txt".into() }, pub_fake.clone());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;

    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::ReviewReady);
    assert_eq!(*pub_fake.calls.lock().unwrap(), 0, "no PR on red verification");
}

#[tokio::test]
async fn pipeline_stops_when_agent_leaves_clean_tree() {
    let db = Db::test_pool().await.unwrap();
    let (_guard, _project_id, task_id) = seed(&db).await;
    // Agent commits everything (incl. the managed CLAUDE.md/.gitignore that worktree
    // setup wrote), leaving a clean working tree → reconcile sees no changes → Stopped.
    let w = worker(
        &db,
        FakeAgent {
            script: "git add -A && git -c user.email=t@t -c user.name=t commit -q -m work".into(),
        },
        FakePublisher::default(),
    );
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;
    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::Stopped);
}

#[tokio::test]
async fn pipeline_fails_after_agent_errors_twice() {
    let db = Db::test_pool().await.unwrap();
    let (_guard, _project_id, task_id) = seed(&db).await;
    let w = worker(&db, FakeAgent { script: "exit 1".into() }, FakePublisher::default());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;
    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::Failed);
}
