use crate::error::AppError;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    /// Initialize the production database at the platform data dir.
    pub async fn init() -> Result<Self, AppError> {
        let path = production_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::internal(format!("create data dir: {e}")))?;
        }
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| AppError::internal(format!("open db: {e}")))?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| AppError::internal(format!("run migrations: {e}")))?;
        Ok(Self { pool })
    }

    /// In-memory pool for tests. Each call returns a fresh independent DB.
    #[cfg(test)]
    pub async fn test_pool() -> Result<Self, AppError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .map_err(|e| AppError::internal(format!("open test db: {e}")))?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| AppError::internal(format!("run migrations: {e}")))?;
        Ok(Self { pool })
    }
}

fn production_db_path() -> Result<PathBuf, AppError> {
    let base = dirs::data_dir().ok_or_else(|| AppError::internal("no platform data dir"))?;
    Ok(base.join("AI Software Studio").join("app.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creates_projects_table() {
        let db = Db::test_pool().await.expect("test pool");
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
        )
        .fetch_one(&db.pool)
        .await
        .expect("query");
        assert_eq!(row.0, 1, "projects table should exist");
    }
}
