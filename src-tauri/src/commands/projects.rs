use crate::{
    error::AppError, models::Project, projects::open::open_project as open_project_impl,
    state::AppState,
};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    state.projects.list().await
}

#[tauri::command]
#[specta::specta]
pub async fn open_project(state: State<'_, AppState>, path: String) -> Result<Project, AppError> {
    open_project_impl(&state.db, &path).await
}
