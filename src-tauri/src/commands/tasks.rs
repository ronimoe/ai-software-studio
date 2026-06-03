use crate::{
    error::AppError,
    models::{CreateTaskRequest, Task},
    state::AppState,
};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_tasks(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<Task>, AppError> {
    state.tasks.list_for_project(&project_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_task(state: State<'_, AppState>, task_id: String) -> Result<Task, AppError> {
    state.tasks.get(&task_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn create_task(
    state: State<'_, AppState>,
    request: CreateTaskRequest,
) -> Result<Task, AppError> {
    state.tasks.create(&request).await
}
