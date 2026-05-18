use crate::{
    core::worktree_lifecycle::create_worktree_lifecycle,
    error::AppError,
    git::worktree_paths::{branch_name, worktree_path},
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
    let repo = std::path::PathBuf::from(&project.path);
    tokio::task::spawn_blocking(move || {
        crate::git::GitService::new().worktree_remove(&repo, &dest)
    })
    .await
    .map_err(|e| AppError::internal(format!("join: {e}")))??;
    state.tasks.clear_worktree(&task.id).await?;
    state.tasks.update_status(&task.id, TaskStatus::Draft).await?;
    Ok(())
}
