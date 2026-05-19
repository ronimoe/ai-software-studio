use crate::{
    core::worktree_lifecycle::create_worktree_lifecycle,
    error::AppError,
    git::worktree_paths::{branch_name, is_within_worktree_root, worktree_path},
    models::{Task, TaskStatus},
    state::AppState,
};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn create_worktree(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    if task.status != TaskStatus::Draft {
        return Err(AppError::invalid_arg(format!(
            "cannot create worktree for task in status {:?}",
            task.status
        )));
    }
    let project = state.projects.get(&task.project_id).await?;

    let branch = branch_name(&task.id);
    let dest = worktree_path(&task.project_id, &task.id)?;

    // Refuse to proceed if dest already exists; clean up via remove first.
    if dest.exists() {
        return Err(AppError::invalid_arg(format!(
            "worktree path already exists: {}",
            dest.display()
        )));
    }

    let repo = std::path::PathBuf::from(&project.path);
    create_worktree_lifecycle(
        &state.git,
        &state.worktree_context,
        &state.tasks,
        &task,
        &repo,
        &branch,
        &dest,
        &project.default_branch,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_worktree(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> {
    let task = state.tasks.get(&task_id).await?;
    let project = state.projects.get(&task.project_id).await?;
    let dest = match task.worktree_path {
        Some(p) => std::path::PathBuf::from(p),
        None => return Err(AppError::invalid_arg("task has no worktree to remove")),
    };

    // Defence-in-depth: a corrupted or imported SQLite row could point
    // `worktree_path` anywhere on disk. Refuse to operate on paths outside the
    // managed worktree root so this command can never be turned into an
    // arbitrary-directory delete.
    if !is_within_worktree_root(&dest) {
        return Err(AppError::invalid_arg(format!(
            "refusing to remove path outside managed worktree root: {}",
            dest.display()
        )));
    }

    let repo = std::path::PathBuf::from(&project.path);
    // Use the branch recorded on the task if present; otherwise derive it the
    // same way `create_worktree` did, so cleanup still gets the dangling ref.
    let branch = task.branch_name.clone().unwrap_or_else(|| branch_name(&task.id));
    tokio::task::spawn_blocking(move || {
        let svc = crate::git::GitService::new();
        svc.worktree_remove(&repo, &dest)?;
        // Best-effort: drop the branch ref too so the user can fully clean up
        // and re-create a worktree for the same task without git complaining.
        svc.branch_delete(&repo, &branch)
    })
    .await
    .map_err(|e| AppError::internal(format!("join: {e}")))??;
    state.tasks.clear_worktree(&task.id).await?;
    state.tasks.update_status(&task.id, TaskStatus::Draft).await?;
    Ok(())
}
