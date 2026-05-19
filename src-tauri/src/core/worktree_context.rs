use crate::{error::AppError, models::Task};
use std::path::Path;

pub const MANAGED_BEGIN: &str = "<!-- aistudio:begin -->";
pub const MANAGED_END: &str = "<!-- aistudio:end -->";

pub struct WorktreeContextService;

impl WorktreeContextService {
    pub fn new() -> Self { Self }

    /// Idempotent: writes task-brief.md into `{worktree}/.aistudio/`, writes/updates
    /// `{worktree}/CLAUDE.md` with a managed section that @-imports the brief,
    /// and ensures `.aistudio/` is in the worktree's `.gitignore`.
    pub fn install(
        &self,
        worktree: &Path,
        task: &Task,
        brief_markdown: &str,
    ) -> Result<(), AppError> {
        let aistudio = worktree.join(".aistudio");
        std::fs::create_dir_all(&aistudio)
            .map_err(|e| AppError::internal(format!("mkdir .aistudio: {e}")))?;

        std::fs::write(aistudio.join("task-brief.md"), brief_markdown)
            .map_err(|e| AppError::internal(format!("write task-brief.md: {e}")))?;

        write_managed_claude_md(worktree, task)?;
        ensure_gitignore_entry(worktree, ".aistudio/")?;

        Ok(())
    }
}

fn write_managed_claude_md(worktree: &Path, task: &Task) -> Result<(), AppError> {
    let path = worktree.join("CLAUDE.md");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let stripped = strip_managed_section(&existing);
    let body = format!(
        "{managed_begin}\n\
        ## AI Software Studio · Task `{id}`\n\
        \n\
        You are working on the task described in `.aistudio/task-brief.md`. Read it first.\n\
        \n\
        @.aistudio/task-brief.md\n\
        \n\
        Rules:\n\
        1. Do not modify files outside the worktree.\n\
        2. Write a failing test before any implementation change.\n\
        3. Ask the user (via the conversation panel) before adding dependencies or modifying sensitive paths.\n\
        4. After implementation, summarize what changed.\n\
        {managed_end}\n",
        managed_begin = MANAGED_BEGIN,
        managed_end = MANAGED_END,
        id = task.id,
    );

    let new_content = if stripped.trim().is_empty() {
        body
    } else {
        format!("{}\n\n{}", stripped.trim_end(), body)
    };
    std::fs::write(&path, new_content)
        .map_err(|e| AppError::internal(format!("write CLAUDE.md: {e}")))?;
    Ok(())
}

fn strip_managed_section(s: &str) -> String {
    let Some(begin) = s.find(MANAGED_BEGIN) else { return s.to_string(); };
    let after_begin = &s[begin..];
    let Some(end_rel) = after_begin.find(MANAGED_END) else { return s.to_string(); };
    let end_abs = begin + end_rel + MANAGED_END.len();
    let mut out = String::with_capacity(s.len());
    out.push_str(&s[..begin]);
    // Consume any trailing whitespace + newline after the managed block.
    let mut idx = end_abs;
    let bytes = s.as_bytes();
    while idx < bytes.len() && (bytes[idx] == b'\n' || bytes[idx] == b' ' || bytes[idx] == b'\t') {
        idx += 1;
    }
    out.push_str(&s[idx..]);
    out
}

fn ensure_gitignore_entry(worktree: &Path, entry: &str) -> Result<(), AppError> {
    let path = worktree.join(".gitignore");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == entry || l.trim() == entry.trim_end_matches('/')) {
        return Ok(());
    }
    let mut new_content = existing.clone();
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(entry);
    new_content.push('\n');
    std::fs::write(&path, new_content)
        .map_err(|e| AppError::internal(format!("write .gitignore: {e}")))?;
    Ok(())
}
