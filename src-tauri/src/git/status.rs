use crate::{error::AppError, models::ChangedFile};
use std::path::Path;

pub fn status(_worktree: &Path) -> Result<Vec<ChangedFile>, AppError> {
    Err(AppError::unimplemented("status"))
}
