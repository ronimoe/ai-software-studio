use crate::{
    db::Db,
    error::AppError,
    models::{AcceptanceCriterion, CreateTaskRequest, RiskLevel, Task, TaskStatus},
};
use uuid::Uuid;

pub struct TaskRepository {
    db: Db,
}

impl TaskRepository {
    pub fn new(db: Db) -> Self { Self { db } }

    pub async fn insert(&self, req: &CreateTaskRequest) -> Result<Task, AppError> {
        let task_id = format!("task-{}", Uuid::new_v4());
        let mut tx = self
            .db
            .pool
            .begin()
            .await
            .map_err(|e| AppError::internal(format!("begin tx: {e}")))?;

        sqlx::query(
            "INSERT INTO tasks
             (id, project_id, title, description, out_of_scope, files_to_touch_hint, selected_engine, status, risk)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'draft', 'unknown')",
        )
        .bind(&task_id)
        .bind(&req.project_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.out_of_scope)
        .bind(&req.files_to_touch_hint)
        .bind(&req.selected_engine)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::internal(format!("insert task: {e}")))?;

        for (i, label) in req.acceptance_criteria.iter().enumerate() {
            sqlx::query(
                "INSERT INTO task_acceptance_criteria (id, task_id, label, satisfied, position) VALUES (?, ?, ?, 0, ?)",
            )
            .bind(format!("ac-{}", Uuid::new_v4()))
            .bind(&task_id)
            .bind(label)
            .bind(i as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("insert ac: {e}")))?;
        }

        for (i, body) in req.constraints.iter().enumerate() {
            sqlx::query(
                "INSERT INTO task_constraints (id, task_id, body, position) VALUES (?, ?, ?, ?)",
            )
            .bind(format!("c-{}", Uuid::new_v4()))
            .bind(&task_id)
            .bind(body)
            .bind(i as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("insert constraint: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::internal(format!("commit tx: {e}")))?;

        self.get(&task_id).await
    }

    pub async fn list_for_project(&self, project_id: &str) -> Result<Vec<Task>, AppError> {
        let ids: Vec<(String,)> = sqlx::query_as(
            "SELECT id FROM tasks WHERE project_id = ? ORDER BY created_at DESC, rowid DESC",
        )
        .bind(project_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("list tasks: {e}")))?;
        let mut out = Vec::with_capacity(ids.len());
        for (id,) in ids {
            out.push(self.get(&id).await?);
        }
        Ok(out)
    }

    pub async fn get(&self, task_id: &str) -> Result<Task, AppError> {
        let row: Option<(String, String, String, String, String, String, Option<String>, String, String, Option<String>, Option<String>, String, Option<String>)> = sqlx::query_as(
            "SELECT id, project_id, title, description, out_of_scope, files_to_touch_hint,
                    selected_engine, status, risk, branch_name, worktree_path, created_at, queued_at
             FROM tasks WHERE id = ?",
        )
        .bind(task_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("get task: {e}")))?;

        let row = row.ok_or_else(|| AppError::not_found(format!("task {task_id} not found")))?;

        let ac_rows: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT id, label, satisfied FROM task_acceptance_criteria WHERE task_id = ? ORDER BY position",
        )
        .bind(task_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("get ac: {e}")))?;

        let constraint_rows: Vec<(String,)> = sqlx::query_as(
            "SELECT body FROM task_constraints WHERE task_id = ? ORDER BY position",
        )
        .bind(task_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("get constraints: {e}")))?;

        Ok(Task {
            id: row.0,
            project_id: row.1,
            title: row.2,
            description: row.3,
            out_of_scope: row.4,
            files_to_touch_hint: row.5,
            selected_engine: row.6,
            status: parse_status(&row.7)?,
            risk: parse_risk(&row.8),
            branch_name: row.9,
            worktree_path: row.10,
            created_at: row.11,
            queued_at: row.12,
            acceptance_criteria: ac_rows
                .into_iter()
                .map(|(id, label, satisfied)| AcceptanceCriterion { id, label, satisfied: satisfied != 0 })
                .collect(),
            constraints: constraint_rows.into_iter().map(|(b,)| b).collect(),
        })
    }

    pub async fn update_status(&self, task_id: &str, status: TaskStatus) -> Result<(), AppError> {
        sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
            .bind(serialize_status(status))
            .bind(task_id)
            .execute(&self.db.pool)
            .await
            .map_err(|e| AppError::internal(format!("update status: {e}")))?;
        Ok(())
    }

    pub async fn set_branch_and_worktree(
        &self,
        task_id: &str,
        branch: &str,
        worktree: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE tasks SET branch_name = ?, worktree_path = ? WHERE id = ?")
            .bind(branch)
            .bind(worktree)
            .bind(task_id)
            .execute(&self.db.pool)
            .await
            .map_err(|e| AppError::internal(format!("set branch+worktree: {e}")))?;
        Ok(())
    }

    pub async fn clear_worktree(&self, task_id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE tasks SET branch_name = NULL, worktree_path = NULL WHERE id = ?")
            .bind(task_id)
            .execute(&self.db.pool)
            .await
            .map_err(|e| AppError::internal(format!("clear worktree: {e}")))?;
        Ok(())
    }
}

fn serialize_status(s: TaskStatus) -> &'static str {
    match s {
        TaskStatus::Draft => "draft",
        TaskStatus::Queued => "queued",
        TaskStatus::WorktreeCreated => "worktreeCreated",
        TaskStatus::Running => "running",
        TaskStatus::NeedsInput => "needsInput",
        TaskStatus::VerificationRunning => "verificationRunning",
        TaskStatus::ReviewReady => "reviewReady",
        TaskStatus::Approved => "approved",
        TaskStatus::PrPrepared => "prPrepared",
        TaskStatus::Done => "done",
        TaskStatus::ChangesRequested => "changesRequested",
        TaskStatus::Rejected => "rejected",
        TaskStatus::Failed => "failed",
        TaskStatus::Stopped => "stopped",
    }
}

fn parse_status(s: &str) -> Result<TaskStatus, AppError> {
    Ok(match s {
        "draft" => TaskStatus::Draft,
        "queued" => TaskStatus::Queued,
        "worktreeCreated" => TaskStatus::WorktreeCreated,
        "running" => TaskStatus::Running,
        "needsInput" => TaskStatus::NeedsInput,
        "verificationRunning" => TaskStatus::VerificationRunning,
        "reviewReady" => TaskStatus::ReviewReady,
        "approved" => TaskStatus::Approved,
        "prPrepared" => TaskStatus::PrPrepared,
        "done" => TaskStatus::Done,
        "changesRequested" => TaskStatus::ChangesRequested,
        "rejected" => TaskStatus::Rejected,
        "failed" => TaskStatus::Failed,
        "stopped" => TaskStatus::Stopped,
        other => return Err(AppError::internal(format!("unknown task status: {other}"))),
    })
}

fn parse_risk(s: &str) -> RiskLevel {
    match s {
        "safe" => RiskLevel::Safe,
        "sensitive" => RiskLevel::Sensitive,
        "dependency" => RiskLevel::Dependency,
        "migration" => RiskLevel::Migration,
        "infra" => RiskLevel::Infra,
        _ => RiskLevel::Unknown,
    }
}
