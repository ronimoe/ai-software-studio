use crate::{
    engines::adapters::claude_code::ClaudeCodeAdapter,
    engines::detection::detect_claude,
    error::AppError,
    models::{Task, TaskStatus},
    state::AppState,
};
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn start_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    match task.status {
        TaskStatus::WorktreeCreated | TaskStatus::Stopped | TaskStatus::Failed => {}
        other => {
            return Err(AppError::invalid_arg(format!(
                "cannot start task in status {other:?}"
            )));
        }
    }
    let worktree = task
        .worktree_path
        .clone()
        .ok_or_else(|| AppError::invalid_arg("task has no worktree; create one first"))?;
    let claude = tokio::task::spawn_blocking(detect_claude)
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))??;
    let binary = claude
        .binary_path
        .ok_or_else(|| AppError::internal("claude binary not found on PATH"))?;

    ClaudeCodeAdapter::start(&state.process, &task.id, &PathBuf::from(&worktree), &binary).await?;
    state.tasks.update_status(&task.id, TaskStatus::Running).await?;
    state.tasks.get(&task.id).await
}

#[tauri::command]
#[specta::specta]
pub async fn stop_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    state.process.stop(&task_id).await?;
    state.tasks.update_status(&task_id, TaskStatus::Stopped).await?;
    state.tasks.get(&task_id).await
}

#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunStatus {
    pub task_id: String,
    pub running: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn get_run_status(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<RunStatus, AppError> {
    Ok(RunStatus {
        task_id: task_id.clone(),
        running: state.process.is_running(&task_id),
    })
}
