use crate::{error::AppError, models::Task};
use std::path::Path;

pub const MANAGED_BEGIN: &str = "<!-- aistudio:begin -->";
pub const MANAGED_END: &str = "<!-- aistudio:end -->";

pub struct WorktreeContextService;

impl WorktreeContextService {
    pub fn new() -> Self { Self }

    pub fn install(&self, _worktree: &Path, _task: &Task, _brief_markdown: &str) -> Result<(), AppError> {
        Err(AppError::unimplemented("install"))
    }
}

pub fn claude_md_content() -> String {
    // Implementation in Task 5.
    String::new()
}
