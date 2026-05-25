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

    pub async fn spawn(
        &self,
        task_id: &str,
        program: &str,
        args: &[&str],
        cwd: &PathBuf,
    ) -> Result<(), AppError> {
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

        // Forward stdout lines.
        if let Some(h) = handle_opt.clone() {
            tokio::spawn(forward_lines(stdout, task_id_owned.clone(), OutputStream::Stdout, h));
        }
        // Forward stderr lines.
        if let Some(h) = handle_opt.clone() {
            tokio::spawn(forward_lines(stderr, task_id_owned.clone(), OutputStream::Stderr, h));
        }
        // Reaper: when the process exits, emit run-exit and unregister.
        let handle_for_reaper = handle_opt;
        let running_for_reaper = running.clone();
        let task_id_for_reaper = task_id_owned;
        tokio::spawn(async move {
            let mut guard = child_arc.lock().await;
            let exit = guard.wait().await;
            running_for_reaper.remove(&task_id_for_reaper);
            if let Some(h) = handle_for_reaper {
                let payload = RunExit {
                    task_id: task_id_for_reaper.clone(),
                    exit_code: exit.as_ref().ok().and_then(|s| s.code()),
                    signaled: exit
                        .as_ref()
                        .ok()
                        .map(|s| s.code().is_none())
                        .unwrap_or(false),
                };
                let _ = h.emit("task-exit", payload);
            }
        });

        Ok(())
    }

    pub async fn stop(&self, task_id: &str) -> Result<(), AppError> {
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
        let payload = OutputLine { task_id: task_id.clone(), stream, text };
        let _ = handle.emit("task-output", payload);
    }
}

#[cfg(test)]
mod tests;
