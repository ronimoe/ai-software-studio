use std::path::Path;

use crate::{
    core::worktree_context::WorktreeContextService,
    error::AppError,
    git::GitService,
    models::{Task, TaskStatus},
    tasks::{brief::render_brief, TaskService},
};

/// Orchestrate worktree creation with LIFO compensating-action cleanup.
///
/// See plan §Architecture / Failure semantics. This is the project's first
/// compensating-action operation: stateful side effects (git worktree, installed
/// files) are rolled back when a later step fails. Pure of `State<'_, AppState>`
/// so it can be unit-tested with real services against a temp repo + in-memory DB.
pub async fn create_worktree_lifecycle(
    _git: &GitService,
    _wt_ctx: &WorktreeContextService,
    tasks: &TaskService,
    task: &Task,
    repo: &Path,
    branch: &str,
    dest: &Path,
) -> Result<Task, AppError> {
    // `_git` and `_wt_ctx` are accepted for API symmetry with AppState; the
    // closures below construct fresh stateless instances inside spawn_blocking
    // because the services don't (yet) carry state and the refs aren't Send.

    // Step 1: worktree_add. No prior state to clean up if this fails.
    spawn_blocking_git({
        let repo = repo.to_path_buf();
        let dest = dest.to_path_buf();
        let branch = branch.to_string();
        move || GitService::new().worktree_add(&repo, &branch, &dest).map(|_| ())
    })
    .await?;

    // Step 2: install. Compensate with worktree_remove on failure.
    let brief = render_brief(task);
    let task_for_install = task.clone();
    let install_outcome = spawn_blocking_git({
        let dest = dest.to_path_buf();
        move || WorktreeContextService::new().install(&dest, &task_for_install, &brief)
    })
    .await;
    if let Err(e) = install_outcome {
        rollback_worktree(repo, dest).await;
        return Err(e);
    }

    // Step 3: update_status. Compensate with worktree_remove on failure.
    if let Err(e) = tasks.update_status(&task.id, TaskStatus::WorktreeCreated).await {
        rollback_worktree(repo, dest).await;
        return Err(e);
    }

    // Step 4: set_branch_and_worktree. Compensate with worktree_remove + status revert.
    if let Err(e) = tasks
        .set_branch_and_worktree(&task.id, branch, dest.to_string_lossy().as_ref())
        .await
    {
        rollback_worktree(repo, dest).await;
        let _ = tasks.update_status(&task.id, TaskStatus::Draft).await;
        return Err(e);
    }

    tasks.get(&task.id).await
}

async fn spawn_blocking_git<T, F>(f: F) -> Result<T, AppError>
where
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))?
}

/// Best-effort cleanup. Errors are intentionally swallowed because we're already
/// returning a failure to the caller; double-failing here would mask the original.
/// `GitService::worktree_remove` is idempotent (see `git::tests::worktree_remove_is_idempotent`).
async fn rollback_worktree(repo: &Path, dest: &Path) {
    let repo = repo.to_path_buf();
    let dest = dest.to_path_buf();
    let _ = tokio::task::spawn_blocking(move || {
        GitService::new().worktree_remove(&repo, &dest)
    })
    .await;
}
