use crate::{error::AppError, models::{ChangeStatus, ChangedFile}};
use std::path::Path;
use std::process::Command;

pub fn status(worktree: &Path) -> Result<Vec<ChangedFile>, AppError> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "-z"])
        .current_dir(worktree)
        .output()
        .map_err(|e| AppError::internal(format!("spawn git status: {e}")))?;
    if !output.status.success() {
        return Err(AppError::invalid_arg(format!(
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_porcelain(&stdout))
}

fn parse_porcelain(stdout: &str) -> Vec<ChangedFile> {
    let mut out = Vec::new();
    // `-z` produces NUL-separated records; each is `XY <path>` and renames have an extra `\0orig`.
    let mut iter = stdout.split('\0').filter(|s| !s.is_empty()).peekable();
    while let Some(record) = iter.next() {
        let bytes = record.as_bytes();
        if bytes.len() < 3 {
            continue;
        }
        let x = bytes[0] as char;
        let y = bytes[1] as char;
        let path = std::str::from_utf8(&bytes[3..]).unwrap_or("").to_string();
        let status = classify(x, y);
        if status == ChangeStatus::Renamed {
            // The next record is the original path; skip it.
            iter.next();
        }
        out.push(ChangedFile {
            path,
            status,
            additions: 0,
            deletions: 0,
        });
    }
    out
}

fn classify(x: char, y: char) -> ChangeStatus {
    match (x, y) {
        ('?', '?') => ChangeStatus::Untracked,
        ('U', _) | (_, 'U') | ('A', 'A') | ('D', 'D') => ChangeStatus::Conflicted,
        ('A', _) => ChangeStatus::Added,
        ('M', _) | (_, 'M') => ChangeStatus::Modified,
        ('D', _) | (_, 'D') => ChangeStatus::Deleted,
        ('R', _) => ChangeStatus::Renamed,
        _ => ChangeStatus::Modified,
    }
}
