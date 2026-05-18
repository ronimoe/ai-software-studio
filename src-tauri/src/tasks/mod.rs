pub mod repository;
pub mod brief;

#[cfg(test)]
mod repository_tests;
#[cfg(test)]
mod brief_tests;

use crate::{error::AppError, fixtures, models::Task};

pub struct TaskService;

impl TaskService {
    pub fn new() -> Self { Self }

    pub async fn list_for_project(&self, project_id: &str) -> Result<Vec<Task>, AppError> {
        Ok(fixtures::tasks_for_project(project_id))
    }

    pub async fn get(&self, task_id: &str) -> Result<Task, AppError> {
        fixtures::tasks_for_project("proj-default")
            .into_iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| AppError::not_found(format!("task {task_id} not found")))
    }
}
