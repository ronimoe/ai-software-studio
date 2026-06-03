use crate::{
    db::Db,
    error::AppError,
    models::{VerificationCheck, VerificationRun, VerificationStatus},
};
use uuid::Uuid;

use super::runner::CheckResult;

pub struct VerificationRepository {
    db: Db,
}

impl VerificationRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn insert_run(
        &self,
        task_id: &str,
        checks: Vec<CheckResult>,
    ) -> Result<VerificationRun, AppError> {
        let run_id = format!("vr-{}", Uuid::new_v4());
        let mut tx = self
            .db
            .pool
            .begin()
            .await
            .map_err(|e| AppError::internal(format!("tx: {e}")))?;
        sqlx::query("INSERT INTO verification_runs (id, task_id) VALUES (?, ?)")
            .bind(&run_id)
            .bind(task_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("insert run: {e}")))?;
        for (i, c) in checks.iter().enumerate() {
            sqlx::query(
                "INSERT INTO verification_checks (id, run_id, kind, status, duration_ms, log_excerpt, position) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(format!("vc-{}", Uuid::new_v4()))
            .bind(&run_id)
            .bind(&c.kind)
            .bind(serialize_status(c.status))
            .bind(c.duration_ms.map(|m| m as i64))
            .bind(c.log_excerpt.as_deref())
            .bind(i as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("insert check: {e}")))?;
        }
        sqlx::query("UPDATE verification_runs SET finished_at = datetime('now') WHERE id = ?")
            .bind(&run_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("finish run: {e}")))?;
        tx.commit()
            .await
            .map_err(|e| AppError::internal(format!("commit: {e}")))?;
        self.get(&run_id).await
    }

    pub async fn list_for_task(&self, task_id: &str) -> Result<Vec<VerificationRun>, AppError> {
        let ids: Vec<(String,)> = sqlx::query_as(
            "SELECT id FROM verification_runs WHERE task_id = ? ORDER BY started_at DESC",
        )
        .bind(task_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("list runs: {e}")))?;
        let mut out = Vec::with_capacity(ids.len());
        for (id,) in ids {
            out.push(self.get(&id).await?);
        }
        Ok(out)
    }

    pub async fn get(&self, id: &str) -> Result<VerificationRun, AppError> {
        let (id, task_id, started): (String, String, String) =
            sqlx::query_as("SELECT id, task_id, started_at FROM verification_runs WHERE id = ?")
                .bind(id)
                .fetch_one(&self.db.pool)
                .await
                .map_err(|e| AppError::internal(format!("get run: {e}")))?;

        let checks: Vec<(String, String, Option<i64>, Option<String>)> = sqlx::query_as(
            "SELECT kind, status, duration_ms, log_excerpt FROM verification_checks WHERE run_id = ? ORDER BY position",
        )
        .bind(&id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("get checks: {e}")))?;

        Ok(VerificationRun {
            id,
            task_id,
            started_at: started,
            checks: checks
                .into_iter()
                .map(|(kind, status, dur, log)| VerificationCheck {
                    kind,
                    status: parse_status(&status),
                    duration_ms: dur.map(|d| d as u32),
                    log_excerpt: log,
                })
                .collect(),
        })
    }
}

fn serialize_status(s: VerificationStatus) -> &'static str {
    match s {
        VerificationStatus::NotRun => "notRun",
        VerificationStatus::Running => "running",
        VerificationStatus::Passed => "passed",
        VerificationStatus::Failed => "failed",
        VerificationStatus::Skipped => "skipped",
        VerificationStatus::Warning => "warning",
    }
}
fn parse_status(s: &str) -> VerificationStatus {
    match s {
        "passed" => VerificationStatus::Passed,
        "failed" => VerificationStatus::Failed,
        "skipped" => VerificationStatus::Skipped,
        "warning" => VerificationStatus::Warning,
        "running" => VerificationStatus::Running,
        _ => VerificationStatus::NotRun,
    }
}
