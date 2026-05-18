pub mod worktree_paths;

#[cfg(test)]
mod tests;

use crate::error::AppError;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitService;

impl GitService {
    pub fn new() -> Self { Self }

    pub fn worktree_add(
        &self,
        _repo_path: &Path,
        _branch: &str,
        _worktree_dir: &Path,
    ) -> Result<PathBuf, AppError> {
        Err(AppError::unimplemented("worktree_add"))
    }

    pub fn worktree_remove(&self, _repo_path: &Path, _worktree_dir: &Path) -> Result<(), AppError> {
        Err(AppError::unimplemented("worktree_remove"))
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
