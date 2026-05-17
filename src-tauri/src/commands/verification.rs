use crate::{error::AppError, models::VerificationRun, state::AppState};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_verification(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<VerificationRun>, AppError> {
    state.verification.list_for_task(&task_id).await
}
