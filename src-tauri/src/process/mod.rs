use crate::error::AppError;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_specta::Event;
use tokio::process::Child;
use tokio::sync::Mutex;

#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TaskOutput {
    pub task_id: String,
    pub stream: OutputStream,
    pub text: String,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum OutputStream { Stdout, Stderr }

#[derive(Clone, serde::Serialize, serde::Deserialize, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct TaskExit {
    pub task_id: String,
    pub exit_code: Option<i32>,
    pub signaled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ExitInfo {
    pub exit_code: Option<i32>,
    pub signaled: bool,
    pub stopped_by_user: bool,
}

pub struct ProcessRunner {
    handle: Mutex<Option<AppHandle>>,
    running: Arc<DashMap<String, Arc<Mutex<Child>>>>,
    exits: Arc<DashMap<String, tokio::sync::watch::Receiver<Option<ExitInfo>>>>,
    stop_requests: Arc<DashMap<String, ()>>,
}

impl Default for ProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRunner {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
            running: Arc::new(DashMap::new()),
            exits: Arc::new(DashMap::new()),
            stop_requests: Arc::new(DashMap::new()),
        }
    }

    pub async fn set_handle(&self, handle: AppHandle) {
        *self.handle.lock().await = Some(handle);
    }

    pub async fn spawn(
        &self,
        task_id: &str,
        program: &str,
        args: &[&str],
        cwd: &PathBuf,
    ) -> Result<(), AppError> {
        // Clear any stale stop marker from a prior run of this id (ids are reused on retry/re-queue).
        self.stop_requests.remove(task_id);
        if self.is_running(task_id) {
            return Err(AppError::invalid_arg(format!(
                "task {task_id} is already running"
            )));
        }
        let mut cmd = tokio::process::Command::new(program);
        cmd.args(args)
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::internal(format!("spawn {program}: {e}")))?;
        let stdout = child.stdout.take().ok_or_else(|| AppError::internal("no stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| AppError::internal("no stderr"))?;

        let task_id_owned = task_id.to_string();
        let handle_opt = self.handle.lock().await.clone();
        let running = self.running.clone();
        let child_arc = Arc::new(Mutex::new(child));
        running.insert(task_id_owned.clone(), child_arc.clone());

        let (exit_tx, exit_rx) = tokio::sync::watch::channel(None);
        self.exits.insert(task_id_owned.clone(), exit_rx);

        // Forward stdout lines.
        if let Some(h) = handle_opt.clone() {
            tokio::spawn(forward_lines(stdout, task_id_owned.clone(), OutputStream::Stdout, h));
        }
        // Forward stderr lines.
        if let Some(h) = handle_opt.clone() {
            tokio::spawn(forward_lines(stderr, task_id_owned.clone(), OutputStream::Stderr, h));
        }
        // Reaper: when the process exits, resolve channel, emit task-exit, and unregister.
        let handle_for_reaper = handle_opt;
        let running_for_reaper = running.clone();
        let stop_requests = self.stop_requests.clone();
        let task_id_for_reaper = task_id_owned;
        tokio::spawn(async move {
            let mut guard = child_arc.lock().await;
            let exit = guard.wait().await;
            running_for_reaper.remove(&task_id_for_reaper);
            let stopped_by_user = stop_requests.remove(&task_id_for_reaper).is_some();
            let exit_code = exit.as_ref().ok().and_then(|s| s.code());
            let signaled = exit.as_ref().ok().map(|s| s.code().is_none()).unwrap_or(false);
            let _ = exit_tx.send(Some(ExitInfo { exit_code, signaled, stopped_by_user }));
            if let Some(h) = handle_for_reaper {
                let payload = TaskExit {
                    task_id: task_id_for_reaper.clone(),
                    exit_code,
                    signaled,
                };
                let _ = payload.emit(&h);
            }
        });

        Ok(())
    }

    /// Await the agent's exit. Returns immediately if the process already exited.
    /// Returns a default (unknown) ExitInfo if the task was never spawned.
    pub async fn wait_for_exit(&self, task_id: &str) -> ExitInfo {
        let mut rx = match self.exits.get(task_id) {
            Some(r) => r.clone(),
            None => return ExitInfo { exit_code: None, signaled: false, stopped_by_user: false },
        };
        loop {
            if let Some(info) = *rx.borrow() {
                return info;
            }
            if rx.changed().await.is_err() {
                return ExitInfo { exit_code: None, signaled: false, stopped_by_user: false };
            }
        }
    }

    pub async fn stop(&self, task_id: &str) -> Result<(), AppError> {
        self.stop_requests.insert(task_id.to_string(), ());
        let Some(entry) = self.running.get(task_id) else { return Ok(()); };
        let child = entry.clone();
        drop(entry); // release the dashmap shard.

        // SIGTERM first.
        #[cfg(unix)]
        {
            if let Some(pid) = child.lock().await.id() {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
        }
        // Wait up to 2 seconds.
        for _ in 0..20 {
            if !self.running.contains_key(task_id) {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // SIGKILL fallback.
        if let Some(entry) = self.running.get(task_id) {
            let _ = entry.lock().await.kill().await;
        }
        Ok(())
    }

    pub fn is_running(&self, task_id: &str) -> bool {
        self.running.contains_key(task_id)
    }
}

async fn forward_lines<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    reader: R,
    task_id: String,
    stream: OutputStream,
    handle: AppHandle,
) {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(text)) = lines.next_line().await {
        let payload = TaskOutput { task_id: task_id.clone(), stream, text };
        let _ = payload.emit(&handle);
    }
}

#[cfg(test)]
mod tests;
