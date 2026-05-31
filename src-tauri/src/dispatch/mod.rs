pub mod seams;
pub mod worker;
pub mod sweep;
// #[cfg(test)] mod worker_tests;  <- added in a later task

use crate::{db::Db, error::AppError};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri_specta::Event;
use tokio::sync::{Mutex, Notify};

/// Snapshot of the dispatcher for the UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DispatchStatus {
    pub running: bool,
    pub queued: u32,
    pub current_task: Option<String>,
}

/// Live narration of what the worker is doing. UI-only; durable state is the task status.
#[derive(Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DispatchEvent {
    pub task_id: String,
    pub stage: String,
    pub outcome: String,
}

/// Shared control surface between commands and the worker.
#[derive(Clone)]
pub struct DispatchHandle {
    pub notify: Arc<Notify>,
    pub paused: Arc<AtomicBool>,
    pub current_task: Arc<Mutex<Option<String>>>,
}

impl DispatchHandle {
    pub fn new(paused: bool) -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            paused: Arc::new(AtomicBool::new(paused)),
            current_task: Arc::new(Mutex::new(None)),
        }
    }
    pub fn wake(&self) {
        self.notify.notify_one();
    }
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        self.wake();
    }
}

const AUTORUN_SCOPE: &str = "global";
const AUTORUN_KEY: &str = "dispatch.autorun";

/// Read the persisted autorun flag (defaults to true / running).
pub async fn get_autorun(db: &Db) -> Result<bool, AppError> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_settings WHERE scope = ? AND key = ?")
            .bind(AUTORUN_SCOPE)
            .bind(AUTORUN_KEY)
            .fetch_optional(&db.pool)
            .await
            .map_err(|e| AppError::internal(format!("get autorun: {e}")))?;
    Ok(row.map(|(v,)| v != "false").unwrap_or(true))
}

/// Persist the autorun flag.
pub async fn set_autorun(db: &Db, on: bool) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app_settings (scope, key, value) VALUES (?, ?, ?)
         ON CONFLICT(scope, key) DO UPDATE SET value = excluded.value",
    )
    .bind(AUTORUN_SCOPE)
    .bind(AUTORUN_KEY)
    .bind(if on { "true" } else { "false" })
    .execute(&db.pool)
    .await
    .map_err(|e| AppError::internal(format!("set autorun: {e}")))?;
    Ok(())
}
