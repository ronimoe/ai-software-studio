use crate::error::AppError;
use std::path::PathBuf;

pub fn worktree_root() -> Result<PathBuf, AppError> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::internal("no platform data dir"))?
        .join("AI Software Studio")
        .join("worktrees");
    std::fs::create_dir_all(&base)
        .map_err(|e| AppError::internal(format!("create worktrees root: {e}")))?;
    Ok(base)
}

pub fn worktree_path(project_id: &str, task_id: &str) -> Result<PathBuf, AppError> {
    Ok(worktree_root()?.join(project_id).join(task_id))
}

pub fn branch_name(task_id: &str) -> String {
    let short = task_id.strip_prefix("task-").unwrap_or(task_id);
    let safe = short.chars().take(8).collect::<String>();
    format!("aistudio/task-{safe}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_name_truncates_uuid_for_readability() {
        let n = branch_name("task-12345678-aaaa-bbbb-cccc-dddddddddddd");
        assert_eq!(n, "aistudio/task-12345678");
    }

    #[test]
    fn branch_name_handles_short_id() {
        let n = branch_name("task-042");
        assert_eq!(n, "aistudio/task-042");
    }

    #[test]
    fn worktree_path_segments_match_layout() {
        let p = worktree_path("proj-1", "task-1").expect("ok");
        let s = p.to_string_lossy();
        assert!(s.contains("AI Software Studio/worktrees/proj-1/task-1"));
    }
}
