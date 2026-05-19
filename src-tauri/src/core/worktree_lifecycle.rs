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
/// files, branch ref) are rolled back when a later step fails. Pure of
/// `State<'_, AppState>` so it can be unit-tested with real services against a
/// temp repo + in-memory DB.
///
/// `base_ref` is the ref the new branch should be created off — pass
/// `&project.default_branch` so a developer's checked-out feature branch can't
/// silently become the base for a task worktree.
#[allow(clippy::too_many_arguments)]
pub async fn create_worktree_lifecycle(
    _git: &GitService,
    _wt_ctx: &WorktreeContextService,
    tasks: &TaskService,
    task: &Task,
    repo: &Path,
    branch: &str,
    dest: &Path,
    base_ref: &str,
) -> Result<Task, AppError> {
    // `_git` and `_wt_ctx` are accepted for API symmetry with AppState; the
    // closures below construct fresh stateless instances inside spawn_blocking
    // because the services don't (yet) carry state and the refs aren't Send.

    // Step 1: worktree_add.
    //
    // Subtle: `git worktree add -b <branch> <dest>` creates the branch ref
    // BEFORE it validates the destination, so a dest-validation failure (e.g.
    // dest already exists as a file) still leaves a dangling branch ref in the
    // user's repo. Run the same LIFO rollback on step-1 failure as on later
    // steps; `branch_delete` and `worktree_remove` are both idempotent so this
    // is safe even when git didn't actually create either side effect.
    let step1 = spawn_blocking_git({
        let repo = repo.to_path_buf();
        let dest = dest.to_path_buf();
        let branch = branch.to_string();
        let base_ref = base_ref.to_string();
        move || {
            GitService::new()
                .worktree_add(&repo, &branch, &dest, Some(&base_ref))
                .map(|_| ())
        }
    })
    .await;
    if let Err(e) = step1 {
        rollback_worktree(repo, dest, branch).await;
        return Err(e);
    }

    // Step 2: install. Compensate with worktree_remove + branch_delete on failure.
    let brief = render_brief(task);
    let task_for_install = task.clone();
    let install_outcome = spawn_blocking_git({
        let dest = dest.to_path_buf();
        move || WorktreeContextService::new().install(&dest, &task_for_install, &brief)
    })
    .await;
    if let Err(e) = install_outcome {
        rollback_worktree(repo, dest, branch).await;
        return Err(e);
    }

    // Step 3: update_status. Compensate with worktree_remove + branch_delete on failure.
    if let Err(e) = tasks.update_status(&task.id, TaskStatus::WorktreeCreated).await {
        rollback_worktree(repo, dest, branch).await;
        return Err(e);
    }

    // Step 4: set_branch_and_worktree. Compensate with worktree_remove +
    // branch_delete + status revert.
    if let Err(e) = tasks
        .set_branch_and_worktree(&task.id, branch, dest.to_string_lossy().as_ref())
        .await
    {
        rollback_worktree(repo, dest, branch).await;
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
///
/// LIFO order: remove the worktree first (so the branch is no longer checked
/// out anywhere), then delete the branch ref. Both git operations are idempotent
/// (see `git::tests::worktree_remove_is_idempotent` and
/// `git::tests::branch_delete_is_idempotent`), so this is safe to call even
/// when one or both side effects never landed.
async fn rollback_worktree(repo: &Path, dest: &Path, branch: &str) {
    let repo = repo.to_path_buf();
    let dest = dest.to_path_buf();
    let branch = branch.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        let svc = GitService::new();
        let _ = svc.worktree_remove(&repo, &dest);
        let _ = svc.branch_delete(&repo, &branch);
        Ok::<(), AppError>(())
    })
    .await;
}
