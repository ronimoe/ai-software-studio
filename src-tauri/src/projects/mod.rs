pub mod repository;

#[cfg(test)]
mod repository_tests;

// The service is rewritten in Task 6. For now keep the stub so other code compiles.
use crate::{error::AppError, fixtures, models::Project};

pub struct ProjectService;

impl ProjectService {
    pub fn new() -> Self { Self }

    pub async fn list(&self) -> Result<Vec<Project>, AppError> {
        Ok(fixtures::projects())
    }
}
