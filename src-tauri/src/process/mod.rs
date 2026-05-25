use crate::error::AppError;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Child;
use tokio::sync::Mutex;

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OutputLine {
    pub task_id: String,
    pub stream: OutputStream,
    pub text: String,
}

#[derive(Clone, Copy, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum OutputStream { Stdout, Stderr }

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunExit {
    pub task_id: String,
    pub exit_code: Option<i32>,
    pub signaled: bool,
}

pub struct ProcessRunner {
    handle: Mutex<Option<AppHandle>>,
    running: Arc<DashMap<String, Arc<Mutex<Child>>>>,
}

impl ProcessRunner {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
            running: Arc::new(DashMap::new()),
        }
    }

    pub async fn set_handle(&self, handle: AppHandle) {
        *self.handle.lock().await = Some(handle);
    }

    /// Spawn `program` with `args` in `cwd`, register it under `task_id`, and stream
    /// stdout/stderr lines to the frontend as `task-output` events. Resolves immediately
    /// after spawn; exit is reported separately via `task-exit` events.
    pub async fn spawn(
        &self,
        _task_id: &str,
        _program: &str,
        _args: &[&str],
        _cwd: &PathBuf,
    ) -> Result<(), AppError> {
        Err(AppError::unimplemented("spawn"))
    }

    /// SIGTERM, wait 2s, SIGKILL if still alive.
    pub async fn stop(&self, _task_id: &str) -> Result<(), AppError> {
        Err(AppError::unimplemented("stop"))
    }

    pub fn is_running(&self, task_id: &str) -> bool {
        self.running.contains_key(task_id)
    }
}

#[cfg(test)]
mod tests;
