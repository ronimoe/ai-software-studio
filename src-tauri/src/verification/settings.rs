use crate::{db::Db, error::AppError, models::VerificationSettings};

pub struct SettingsRepository { db: Db }

impl SettingsRepository {
    pub fn new(db: Db) -> Self { Self { db } }

    pub async fn get_for_project(&self, project_id: &str) -> Result<VerificationSettings, AppError> {
        let scope = scope(project_id);
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT key, value FROM app_settings WHERE scope = ?",
        )
        .bind(&scope)
        .fetch_all(&self.db.pool).await
        .map_err(|e| AppError::internal(format!("get settings: {e}")))?;
        if rows.is_empty() {
            return Ok(VerificationSettings::default());
        }
        let mut s = VerificationSettings {
            install: None, typecheck: None, lint: None, test: None, build: None,
        };
        for (k, v) in rows {
            let v = if v.is_empty() { None } else { Some(v) };
            match k.as_str() {
                "verification.install" => s.install = v,
                "verification.typecheck" => s.typecheck = v,
                "verification.lint" => s.lint = v,
                "verification.test" => s.test = v,
                "verification.build" => s.build = v,
                _ => {}
            }
        }
        Ok(s)
    }

    pub async fn set_for_project(&self, project_id: &str, settings: &VerificationSettings) -> Result<(), AppError> {
        let scope = scope(project_id);
        let mut tx = self.db.pool.begin().await
            .map_err(|e| AppError::internal(format!("tx: {e}")))?;
        sqlx::query("DELETE FROM app_settings WHERE scope = ?")
            .bind(&scope).execute(&mut *tx).await
            .map_err(|e| AppError::internal(format!("clear: {e}")))?;
        for (key, value) in [
            ("verification.install", &settings.install),
            ("verification.typecheck", &settings.typecheck),
            ("verification.lint", &settings.lint),
            ("verification.test", &settings.test),
            ("verification.build", &settings.build),
        ] {
            sqlx::query("INSERT INTO app_settings (scope, key, value) VALUES (?, ?, ?)")
                .bind(&scope).bind(key).bind(value.clone().unwrap_or_default())
                .execute(&mut *tx).await
                .map_err(|e| AppError::internal(format!("insert: {e}")))?;
        }
        tx.commit().await.map_err(|e| AppError::internal(format!("commit: {e}")))?;
        Ok(())
    }
}

fn scope(project_id: &str) -> String { format!("project:{project_id}") }
