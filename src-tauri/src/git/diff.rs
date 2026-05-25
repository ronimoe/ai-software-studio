use crate::error::AppError;
use std::path::Path;
use std::process::Command;

pub fn diff(worktree: &Path, path: &str) -> Result<String, AppError> {
    // `git diff --no-color HEAD -- <path>` covers tracked changes.
    // For untracked files, briefly mark them with `git add --intent-to-add` so they appear
    // in the diff, then unstage them by resetting (intent-to-add doesn't actually stage content).
    let is_untracked = is_untracked(worktree, path)?;
    if is_untracked {
        let _ = Command::new("git")
            .args(["add", "--intent-to-add", "--", path])
            .current_dir(worktree)
            .output()
            .map_err(|e| AppError::internal(format!("intent-to-add: {e}")))?;
    }

    let output = Command::new("git")
        .args(["diff", "--no-color", "HEAD", "--", path])
        .current_dir(worktree)
        .output()
        .map_err(|e| AppError::internal(format!("spawn git diff: {e}")))?;

    if is_untracked {
        // Reset intent-to-add so the worktree state is unchanged.
        let _ = Command::new("git")
            .args(["reset", "--", path])
            .current_dir(worktree)
            .output();
    }

    if !output.status.success() {
        return Err(AppError::invalid_arg(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn is_untracked(worktree: &Path, path: &str) -> Result<bool, AppError> {
    let out = Command::new("git")
        .args(["ls-files", "--error-unmatch", "--", path])
        .current_dir(worktree)
        .output()
        .map_err(|e| AppError::internal(format!("ls-files: {e}")))?;
    Ok(!out.status.success())
}
