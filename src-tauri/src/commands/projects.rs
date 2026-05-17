use crate::{error::AppError, models::Project, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    state.projects.list().await
}
