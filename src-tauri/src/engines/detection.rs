use crate::{error::AppError, models::EngineStatus};

pub fn detect_claude() -> Result<EngineStatus, AppError> {
    Err(AppError::unimplemented("detect_claude"))
}
