use crate::{error::AppError, process::ProcessRunner};
use std::path::Path;

/// Resolves the command to spawn for a task's agent run. A seam so tests can
/// inject a fake short-lived process instead of the real `claude` binary.
pub trait AgentLauncher: Send + Sync {
    /// Returns `(program, args)` to spawn, or Err if the engine is unavailable.
    fn command(&self, task_id: &str) -> Result<(String, Vec<String>), AppError>;
}

pub struct ClaudeAgentLauncher;

impl AgentLauncher for ClaudeAgentLauncher {
    fn command(&self, task_id: &str) -> Result<(String, Vec<String>), AppError> {
        let claude = crate::engines::detection::detect_claude()?;
        let binary = claude
            .binary_path
            .ok_or_else(|| AppError::internal("claude binary not found on PATH"))?;
        let prompt = crate::engines::adapters::claude_code::build_prompt(task_id);
        Ok((binary, vec!["--print".to_string(), prompt]))
    }
}

/// Pushes a branch and opens a PR. A seam so tests don't shell out to real `gh`.
pub trait PrPublisher: Send + Sync {
    fn push_branch(&self, repo: &Path, branch: &str) -> Result<(), AppError>;
    fn create_pr(
        &self,
        repo: &Path,
        title: &str,
        body: &str,
        base: &str,
        draft: bool,
    ) -> Result<String, AppError>;
}

pub struct GhPublisher;

impl PrPublisher for GhPublisher {
    fn push_branch(&self, repo: &Path, branch: &str) -> Result<(), AppError> {
        crate::engines::github::push_branch(repo, branch)
    }
    fn create_pr(
        &self,
        repo: &Path,
        title: &str,
        body: &str,
        base: &str,
        draft: bool,
    ) -> Result<String, AppError> {
        crate::engines::github::create_pr(repo, title, body, base, draft)
    }
}

/// Spawn a task's agent through the given launcher + runner.
pub async fn launch_agent(
    launcher: &dyn AgentLauncher,
    runner: &ProcessRunner,
    task_id: &str,
    worktree: &Path,
) -> Result<(), AppError> {
    let (program, args) = launcher.command(task_id)?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    runner
        .spawn(task_id, &program, &arg_refs, &worktree.to_path_buf())
        .await
}
