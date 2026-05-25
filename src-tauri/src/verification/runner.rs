use crate::{error::AppError, models::VerificationStatus};
use std::path::Path;

pub struct CheckResult {
    pub kind: String,
    pub status: VerificationStatus,
    pub duration_ms: Option<u32>,
    pub log_excerpt: Option<String>,
}

pub async fn run_single(_worktree: &Path, _kind: &str, _command: &str) -> Result<CheckResult, AppError> {
    Err(AppError::unimplemented("run_single"))
}
