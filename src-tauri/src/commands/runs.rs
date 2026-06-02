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

/// Pure guard for stop eligibility — extracted so it's unit-testable without `AppState`.
/// Only a task with a live process (Running or VerificationRunning) can be stopped;
/// stopping anything else would incorrectly stamp it `Stopped`.
pub(crate) fn stop_eligibility(status: TaskStatus) -> Result<(), AppError> {
    match status {
        TaskStatus::Running | TaskStatus::VerificationRunning => Ok(()),
        other => Err(AppError::invalid_arg(format!(
            "cannot stop task in status {other:?}"
        ))),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn stop_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    stop_eligibility(task.status)?;
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

#[tauri::command]
#[specta::specta]
pub async fn reconcile_after_exit(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    if task.status != TaskStatus::Running {
        return Ok(task);
    }
    let wt = match task.worktree_path.clone() {
        Some(p) => p,
        None => {
            state.tasks.update_status(&task.id, TaskStatus::Stopped).await?;
            return state.tasks.get(&task.id).await;
        }
    };
    let path = std::path::PathBuf::from(wt);
    let changed: Vec<crate::models::ChangedFile> =
        tokio::task::spawn_blocking(move || crate::git::status::status(&path))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))??;
    let new_status = if changed.is_empty() {
        TaskStatus::Stopped
    } else {
        TaskStatus::ReviewReady
    };
    state.tasks.update_status(&task.id, new_status).await?;
    state.tasks.get(&task.id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_stopping_a_live_task() {
        assert!(stop_eligibility(TaskStatus::Running).is_ok());
        assert!(stop_eligibility(TaskStatus::VerificationRunning).is_ok());
    }

    #[test]
    fn rejects_stopping_a_task_without_a_live_process() {
        assert!(stop_eligibility(TaskStatus::Draft).is_err());
        assert!(stop_eligibility(TaskStatus::Queued).is_err());
        assert!(stop_eligibility(TaskStatus::ReviewReady).is_err());
        assert!(stop_eligibility(TaskStatus::Stopped).is_err());
        assert!(stop_eligibility(TaskStatus::Done).is_err());
    }
}
