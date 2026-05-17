use crate::{error::AppError, fixtures, models::EngineStatus};

pub struct EngineService;

impl EngineService {
    pub fn new() -> Self { Self }

    pub async fn list(&self) -> Result<Vec<EngineStatus>, AppError> {
        Ok(fixtures::engines())
    }

    /// Phase 1 stub. Replaced with real `which`/`--version` shelling in a later phase.
    pub async fn detect(&self) -> Result<Vec<EngineStatus>, AppError> {
        Ok(fixtures::engines())
    }
}
