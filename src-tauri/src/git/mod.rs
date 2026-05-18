pub mod worktree_paths;

#[cfg(test)]
mod tests;

use crate::error::AppError;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitService;

impl GitService {
    pub fn new() -> Self { Self }

    /// Create a new branch (off the repo's HEAD) and add a worktree pointing at it.
    pub fn worktree_add(
        &self,
        repo_path: &Path,
        branch: &str,
        worktree_dir: &Path,
    ) -> Result<PathBuf, AppError> {
        if !repo_path.join(".git").exists() && !repo_path.join("HEAD").exists() {
            return Err(AppError::invalid_arg(format!(
                "not a git repo: {}",
                repo_path.display()
            )));
        }
        if let Some(parent) = worktree_dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::internal(format!("create worktree parent: {e}")))?;
        }
        run_git(
            repo_path,
            &[
                "worktree",
                "add",
                "-b",
                branch,
                worktree_dir.to_str().ok_or_else(|| AppError::internal("worktree path utf-8"))?,
            ],
        )?;
        Ok(worktree_dir.to_path_buf())
    }

    pub fn worktree_remove(&self, repo_path: &Path, worktree_dir: &Path) -> Result<(), AppError> {
        // Use --force so a worktree with uncommitted changes can still be cleaned up.
        let _ = run_git(
            repo_path,
            &[
                "worktree",
                "remove",
                "--force",
                worktree_dir.to_str().ok_or_else(|| AppError::internal("worktree path utf-8"))?,
            ],
        );
        // Belt-and-suspenders: ensure the directory is gone even if git refuses.
        if worktree_dir.exists() {
            std::fs::remove_dir_all(worktree_dir)
                .map_err(|e| AppError::internal(format!("rm worktree: {e}")))?;
        }
        Ok(())
    }
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String, AppError> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| AppError::internal(format!("spawn git: {e}")))?;
    if !out.status.success() {
        return Err(AppError::internal(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
