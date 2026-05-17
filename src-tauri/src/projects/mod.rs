use crate::{error::AppError, fixtures, models::Project};

pub struct ProjectService;

impl ProjectService {
    pub fn new() -> Self { Self }

    pub async fn list(&self) -> Result<Vec<Project>, AppError> {
        Ok(fixtures::projects())
    }
}
