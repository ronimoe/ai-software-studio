use crate::{
    error::AppError,
    process::ProcessRunner,
};
use std::path::PathBuf;

pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn id() -> &'static str { "claude-code" }

    /// Spawn `claude` in the worktree. The worktree's `CLAUDE.md` carries the task brief
    /// (see Plan 3 / `WorktreeContextService`), so the argv stays minimal — just a `--print`
    /// prompt instructing the agent to read CLAUDE.md and proceed.
    pub async fn start(
        runner: &ProcessRunner,
        task_id: &str,
        worktree: &PathBuf,
        binary_path: &str,
    ) -> Result<(), AppError> {
        let prompt = build_prompt(task_id);
        runner
            .spawn(
                task_id,
                binary_path,
                &["--print", &prompt],
                worktree,
            )
            .await
    }
}

pub(crate) fn build_prompt(task_id: &str) -> String {
    format!(
        "You are working inside a git worktree for AI Software Studio task `{task_id}`.\n\
         Read `CLAUDE.md` (in the working directory) before doing anything else.\n\
         Follow the rules and the linked `.aistudio/task-brief.md`.\n\
         Write a failing test before any implementation. Do not modify files outside this worktree.\n\
         When you are done, summarize what changed and stop."
    )
}
