pub mod runner;
pub mod repository;
pub mod settings;

#[cfg(test)] mod runner_tests;
#[cfg(test)] mod repository_tests;
#[cfg(test)] mod settings_tests;

use crate::{error::AppError, fixtures, models::VerificationRun};

pub struct VerificationService;

impl VerificationService {
    pub fn new() -> Self { Self }

    pub async fn list_for_task(&self, task_id: &str) -> Result<Vec<VerificationRun>, AppError> {
        Ok(fixtures::verification_for_task(task_id))
    }
}
