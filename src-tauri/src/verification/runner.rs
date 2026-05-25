use crate::{error::AppError, models::VerificationStatus};
use std::path::Path;
use std::time::Instant;
use tokio::io::AsyncReadExt;

const EXCERPT_BYTES: usize = 4096;

pub struct CheckResult {
    pub kind: String,
    pub status: VerificationStatus,
    pub duration_ms: Option<u32>,
    pub log_excerpt: Option<String>,
}

pub async fn run_single(worktree: &Path, kind: &str, command: &str) -> Result<CheckResult, AppError> {
    if command.trim().is_empty() {
        return Ok(CheckResult {
            kind: kind.to_string(),
            status: VerificationStatus::Skipped,
            duration_ms: None,
            log_excerpt: None,
        });
    }
    let start = Instant::now();
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(worktree)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::internal(format!("spawn {kind}: {e}")))?;

    let mut stdout = child.stdout.take().ok_or_else(|| AppError::internal("no stdout"))?;
    let mut stderr = child.stderr.take().ok_or_else(|| AppError::internal("no stderr"))?;
    let mut buf = Vec::with_capacity(EXCERPT_BYTES * 2);

    let mut out_tmp = [0u8; 1024];
    let mut err_tmp = [0u8; 1024];
    let mut stdout_done = false;
    let mut stderr_done = false;
    while !(stdout_done && stderr_done) {
        tokio::select! {
            r = stdout.read(&mut out_tmp), if !stdout_done => match r {
                Ok(0) => stdout_done = true,
                Ok(n) => { buf.extend_from_slice(&out_tmp[..n]); cap_buf(&mut buf); }
                Err(_) => stdout_done = true,
            },
            r = stderr.read(&mut err_tmp), if !stderr_done => match r {
                Ok(0) => stderr_done = true,
                Ok(n) => { buf.extend_from_slice(&err_tmp[..n]); cap_buf(&mut buf); }
                Err(_) => stderr_done = true,
            },
        }
    }

    let exit = child
        .wait()
        .await
        .map_err(|e| AppError::internal(format!("wait {kind}: {e}")))?;
    let elapsed = start.elapsed().as_millis() as u32;

    let status = if exit.success() {
        VerificationStatus::Passed
    } else {
        VerificationStatus::Failed
    };

    let excerpt = if buf.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buf).to_string())
    };

    Ok(CheckResult {
        kind: kind.to_string(),
        status,
        duration_ms: Some(elapsed),
        log_excerpt: excerpt,
    })
}

fn cap_buf(buf: &mut Vec<u8>) {
    if buf.len() > EXCERPT_BYTES {
        let drop_n = buf.len() - EXCERPT_BYTES;
        buf.drain(..drop_n);
    }
}
