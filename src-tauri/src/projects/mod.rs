pub mod open;
pub mod repository;

#[cfg(test)]
mod repository_tests;
#[cfg(test)]
mod open_tests;

use crate::{db::Db, error::AppError, models::Project};
use repository::ProjectRepository;

pub struct ProjectService {
    repo: ProjectRepository,
}

impl ProjectService {
    pub fn new(db: Db) -> Self {
        Self { repo: ProjectRepository::new(db) }
    }

    pub async fn list(&self) -> Result<Vec<Project>, AppError> {
        self.repo.list().await
    }

    pub async fn get(&self, id: &str) -> Result<Project, AppError> {
        self.repo.get(id).await
    }

    pub async fn insert(&self, project: &Project) -> Result<(), AppError> {
        self.repo.insert(project).await
    }
}
