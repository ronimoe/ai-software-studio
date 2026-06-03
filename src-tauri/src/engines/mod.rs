pub mod adapters;
pub mod detection;
pub mod github;

#[cfg(test)]
mod detection_tests;
#[cfg(test)]
mod github_tests;

use crate::{error::AppError, models::EngineStatus};

pub struct EngineService;

impl Default for EngineService {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list(&self) -> Result<Vec<EngineStatus>, AppError> {
        // For v0.1, list and detect return the same thing.
        self.detect().await
    }

    pub async fn detect(&self) -> Result<Vec<EngineStatus>, AppError> {
        let claude = tokio::task::spawn_blocking(detection::detect_claude)
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))??;
        Ok(vec![claude])
    }
}
