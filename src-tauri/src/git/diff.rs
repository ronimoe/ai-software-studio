use crate::error::AppError;
use std::path::Path;

pub fn diff(_worktree: &Path, _path: &str) -> Result<String, AppError> {
    Err(AppError::unimplemented("diff"))
}
