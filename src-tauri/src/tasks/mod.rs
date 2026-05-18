pub mod brief;
pub mod repository;

#[cfg(test)]
mod brief_tests;
#[cfg(test)]
mod repository_tests;

use crate::{
    db::Db,
    error::AppError,
    models::{CreateTaskRequest, Task, TaskStatus},
};
use repository::TaskRepository;

pub struct TaskService {
    repo: TaskRepository,
}

impl TaskService {
    pub fn new(db: Db) -> Self {
        Self { repo: TaskRepository::new(db) }
    }

    pub async fn list_for_project(&self, project_id: &str) -> Result<Vec<Task>, AppError> {
        self.repo.list_for_project(project_id).await
    }

    pub async fn get(&self, task_id: &str) -> Result<Task, AppError> {
        self.repo.get(task_id).await
    }

    pub async fn create(&self, req: &CreateTaskRequest) -> Result<Task, AppError> {
        let task = self.repo.insert(req).await?;
        brief::write_brief(&task)?;
        Ok(task)
    }

    pub async fn update_status(&self, task_id: &str, status: TaskStatus) -> Result<(), AppError> {
        self.repo.update_status(task_id, status).await
    }

    pub async fn set_branch_and_worktree(
        &self,
        task_id: &str,
        branch: &str,
        worktree: &str,
    ) -> Result<(), AppError> {
        self.repo.set_branch_and_worktree(task_id, branch, worktree).await
    }

    pub async fn clear_worktree(&self, task_id: &str) -> Result<(), AppError> {
        self.repo.clear_worktree(task_id).await
    }
}
