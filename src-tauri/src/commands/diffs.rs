use crate::{
    error::AppError,
    models::ChangedFile,
    state::AppState,
};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn get_changed_files(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<ChangedFile>, AppError> {
    let task = state.tasks.get(&task_id).await?;
    let wt = task
        .worktree_path
        .ok_or_else(|| AppError::invalid_arg("task has no worktree"))?;
    let path = PathBuf::from(wt);
    tokio::task::spawn_blocking(move || crate::git::status::status(&path))
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))?
}

#[tauri::command]
#[specta::specta]
pub async fn get_file_diff(
    state: State<'_, AppState>,
    task_id: String,
    path: String,
) -> Result<String, AppError> {
    let task = state.tasks.get(&task_id).await?;
    let wt = task
        .worktree_path
        .ok_or_else(|| AppError::invalid_arg("task has no worktree"))?;
    let wt_path = PathBuf::from(wt);
    tokio::task::spawn_blocking(move || crate::git::diff::diff(&wt_path, &path))
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))?
}
