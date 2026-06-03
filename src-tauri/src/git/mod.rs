pub mod worktree_paths;
pub mod status;
pub mod diff;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod status_tests;
#[cfg(test)]
mod diff_tests;

use crate::error::AppError;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitService;

impl Default for GitService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitService {
    pub fn new() -> Self { Self }

    pub fn status(&self, worktree: &std::path::Path) -> Result<Vec<crate::models::ChangedFile>, AppError> {
        status::status(worktree)
    }

    pub fn diff(&self, worktree: &std::path::Path, path: &str) -> Result<String, AppError> {
        diff::diff(worktree, path)
    }

    /// Create a new branch and add a worktree pointing at it.
    ///
    /// When `base_ref` is `Some`, the new branch is created off that ref (e.g.
    /// the project's default branch). When `None`, git uses the parent repo's
    /// current HEAD — fine for tests, but the production lifecycle should always
    /// pass `Some(&project.default_branch)` so a developer's checked-out feature
    /// branch can't silently become the base for a task worktree.
    pub fn worktree_add(
        &self,
        repo_path: &Path,
        branch: &str,
        worktree_dir: &Path,
        base_ref: Option<&str>,
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
        let dest_str =
            worktree_dir.to_str().ok_or_else(|| AppError::internal("worktree path utf-8"))?;
        let mut args: Vec<&str> = vec!["worktree", "add", "-b", branch, dest_str];
        if let Some(base) = base_ref {
            args.push(base);
        }
        run_git(repo_path, &args)?;
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

    /// Best-effort branch deletion. Idempotent: a missing branch is not an error,
    /// because rollback paths must be safe to call regardless of whether the
    /// branch was actually created. Mirrors `worktree_remove`'s contract.
    pub fn branch_delete(&self, repo_path: &Path, branch: &str) -> Result<(), AppError> {
        let out = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AppError::internal(format!("spawn git: {e}")))?;
        if out.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
        // Swallow "not found" so callers can treat this as idempotent.
        if stderr.contains("not found") || stderr.contains("no such branch") {
            return Ok(());
        }
        Err(AppError::internal(format!(
            "git branch -D {}: {}",
            branch,
            String::from_utf8_lossy(&out.stderr).trim()
        )))
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
