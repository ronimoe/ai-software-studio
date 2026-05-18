use crate::{db::Db, error::AppError, models::Project};

pub struct ProjectRepository {
    db: Db,
}

impl ProjectRepository {
    pub fn new(db: Db) -> Self { Self { db } }

    pub async fn insert(&self, project: &Project) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO projects (id, name, path, default_branch) VALUES (?, ?, ?, ?)",
        )
        .bind(&project.id)
        .bind(&project.name)
        .bind(&project.path)
        .bind(&project.default_branch)
        .execute(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("insert project: {e}")))?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Project>, AppError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT id, name, path, default_branch FROM projects ORDER BY created_at DESC",
        )
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("list projects: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|(id, name, path, default_branch)| Project { id, name, path, default_branch })
            .collect())
    }

    pub async fn get(&self, id: &str) -> Result<Project, AppError> {
        let row: Option<(String, String, String, String)> = sqlx::query_as(
            "SELECT id, name, path, default_branch FROM projects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("get project: {e}")))?;
        row.map(|(id, name, path, default_branch)| Project { id, name, path, default_branch })
            .ok_or_else(|| AppError::not_found(format!("project {id} not found")))
    }
}
