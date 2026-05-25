use crate::{
    error::AppError,
    models::{VerificationRun, VerificationSettings},
    state::AppState,
};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_verification(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<VerificationRun>, AppError> {
    state.verification.list_for_task(&task_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn run_verification(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<VerificationRun, AppError> {
    let task = state.tasks.get(&task_id).await?;
    let worktree = task
        .worktree_path
        .clone()
        .ok_or_else(|| AppError::invalid_arg("task has no worktree; create one first"))?;
    state
        .verification
        .run_for_task(&task.project_id, &task.id, &PathBuf::from(worktree))
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn get_verification_settings(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<VerificationSettings, AppError> {
    state.verification.get_settings(&project_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_verification_settings(
    state: State<'_, AppState>,
    project_id: String,
    settings: VerificationSettings,
) -> Result<(), AppError> {
    state.verification.set_settings(&project_id, &settings).await
}
