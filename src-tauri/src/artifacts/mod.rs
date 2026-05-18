use crate::error::AppError;
use std::path::PathBuf;

/// Durable artifact path for a given task.
/// `~/Library/Application Support/AI Software Studio/projects/{project_id}/tasks/{task_id}/artifacts/`
/// on macOS; the platform equivalent elsewhere.
pub fn artifact_dir(project_id: &str, task_id: &str) -> Result<PathBuf, AppError> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::internal("no platform data dir"))?
        .join("AI Software Studio")
        .join("projects")
        .join(project_id)
        .join("tasks")
        .join(task_id)
        .join("artifacts");
    std::fs::create_dir_all(&base)
        .map_err(|e| AppError::internal(format!("create artifact dir: {e}")))?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_dir_contains_segments() {
        let d = artifact_dir("proj-x", "task-y").expect("dir");
        let s = d.to_string_lossy();
        assert!(s.contains("AI Software Studio"));
        assert!(s.contains("projects/proj-x"));
        assert!(s.contains("tasks/task-y"));
        assert!(s.ends_with("artifacts"));
    }
}
