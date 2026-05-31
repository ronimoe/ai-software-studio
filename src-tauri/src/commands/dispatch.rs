use crate::{
    dispatch::{set_autorun, DispatchStatus},
    error::AppError,
    models::{Task, TaskStatus},
    state::AppState,
};
use tauri::State;

/// Pure guard for queue eligibility — extracted so it's unit-testable without `AppState`.
pub(crate) fn enqueue_eligibility(status: TaskStatus, engine: Option<&str>) -> Result<(), AppError> {
    match status {
        TaskStatus::Draft
        | TaskStatus::Stopped
        | TaskStatus::Failed
        | TaskStatus::ChangesRequested => {}
        other => {
            return Err(AppError::invalid_arg(format!(
                "cannot queue task in status {other:?}"
            )))
        }
    }
    let engine = engine.unwrap_or("claude-code");
    if engine != "claude-code" {
        return Err(AppError::invalid_arg(format!(
            "engine '{engine}' has no adapter yet; only claude-code is dispatchable"
        )));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn enqueue_task(state: State<'_, AppState>, task_id: String) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    enqueue_eligibility(task.status, task.selected_engine.as_deref())?;
    state.tasks.enqueue(&task_id).await?;
    state.dispatch.wake();
    state.tasks.get(&task_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn dequeue_task(state: State<'_, AppState>, task_id: String) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    if task.status != TaskStatus::Queued {
        return Err(AppError::invalid_arg("task is not queued"));
    }
    state.tasks.dequeue(&task_id).await?;
    state.tasks.get(&task_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_dispatch_status(state: State<'_, AppState>) -> Result<DispatchStatus, AppError> {
    let queued = state.tasks.count_queued().await?;
    let current_task = state.dispatch.current_task.lock().await.clone();
    Ok(DispatchStatus {
        running: !state.dispatch.is_paused(),
        queued,
        current_task,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn pause_dispatch(state: State<'_, AppState>) -> Result<(), AppError> {
    state.dispatch.pause();
    set_autorun(&state.db, false).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn resume_dispatch(state: State<'_, AppState>) -> Result<(), AppError> {
    state.dispatch.resume();
    set_autorun(&state.db, true).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_draft_stopped_and_null_engine() {
        assert!(enqueue_eligibility(TaskStatus::Draft, Some("claude-code")).is_ok());
        assert!(enqueue_eligibility(TaskStatus::Draft, None).is_ok()); // null → claude-code
        assert!(enqueue_eligibility(TaskStatus::Stopped, Some("claude-code")).is_ok());
        assert!(enqueue_eligibility(TaskStatus::ChangesRequested, Some("claude-code")).is_ok());
    }

    #[test]
    fn rejects_codex_engine() {
        assert!(enqueue_eligibility(TaskStatus::Draft, Some("codex-cli")).is_err());
    }

    #[test]
    fn rejects_non_requeuable_status() {
        assert!(enqueue_eligibility(TaskStatus::Running, Some("claude-code")).is_err());
        assert!(enqueue_eligibility(TaskStatus::Queued, Some("claude-code")).is_err());
    }
}
