pub mod repository;
pub mod runner;
pub mod settings;

#[cfg(test)]
mod repository_tests;
#[cfg(test)]
mod runner_tests;
#[cfg(test)]
mod settings_tests;

use crate::{
    db::Db,
    error::AppError,
    models::{VerificationRun, VerificationSettings},
};
use repository::VerificationRepository;
use runner::{run_single, CheckResult};
use settings::SettingsRepository;
use std::path::Path;

pub struct VerificationService {
    repo: VerificationRepository,
    settings: SettingsRepository,
}

impl VerificationService {
    pub fn new(db: Db) -> Self {
        Self {
            repo: VerificationRepository::new(db.clone()),
            settings: SettingsRepository::new(db),
        }
    }

    pub async fn list_for_task(&self, task_id: &str) -> Result<Vec<VerificationRun>, AppError> {
        self.repo.list_for_task(task_id).await
    }

    pub async fn get_settings(&self, project_id: &str) -> Result<VerificationSettings, AppError> {
        self.settings.get_for_project(project_id).await
    }

    pub async fn set_settings(
        &self,
        project_id: &str,
        s: &VerificationSettings,
    ) -> Result<(), AppError> {
        self.settings.set_for_project(project_id, s).await
    }

    pub async fn run_for_task(
        &self,
        project_id: &str,
        task_id: &str,
        worktree: &Path,
    ) -> Result<VerificationRun, AppError> {
        let s = self.settings.get_for_project(project_id).await?;
        let plan: Vec<(&str, Option<String>)> = vec![
            ("install", s.install),
            ("typecheck", s.typecheck),
            ("lint", s.lint),
            ("test", s.test),
            ("build", s.build),
        ];
        let mut results: Vec<CheckResult> = Vec::with_capacity(plan.len());
        for (kind, cmd) in plan {
            let res = run_single(worktree, kind, cmd.as_deref().unwrap_or("")).await?;
            results.push(res);
        }
        self.repo.insert_run(task_id, results).await
    }
}
