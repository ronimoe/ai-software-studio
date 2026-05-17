use crate::{error::AppError, models::EngineStatus, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_engines(state: State<'_, AppState>) -> Result<Vec<EngineStatus>, AppError> {
    state.engines.list().await
}

#[tauri::command]
#[specta::specta]
pub async fn detect_engines(state: State<'_, AppState>) -> Result<Vec<EngineStatus>, AppError> {
    state.engines.detect().await
}
