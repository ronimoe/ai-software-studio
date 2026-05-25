use crate::{error::AppError, models::{GitHubAuthStatus, GitHubStatus}};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

pub fn detect() -> Result<GitHubStatus, AppError> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let Some(bin) = which_in("gh", &path_var) else {
        return Ok(GitHubStatus {
            auth: GitHubAuthStatus::NotInstalled,
            binary_path: None,
            account: None,
        });
    };
    let auth_out = Command::new(&bin)
        .args(["auth", "status"])
        .output()
        .map_err(|e| AppError::internal(format!("gh auth status: {e}")))?;

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&auth_out.stdout),
        String::from_utf8_lossy(&auth_out.stderr)
    );
    let account = parse_account(&combined);

    let auth = if auth_out.status.success() {
        GitHubAuthStatus::Authed
    } else {
        GitHubAuthStatus::NotAuthed
    };

    Ok(GitHubStatus {
        auth,
        binary_path: Some(bin),
        account,
    })
}

pub fn push_branch(repo: &Path, branch: &str) -> Result<(), AppError> {
    let out = Command::new("git")
        .args(["push", "-u", "origin", branch])
        .current_dir(repo)
        .output()
        .map_err(|e| AppError::internal(format!("git push: {e}")))?;
    if !out.status.success() {
        return Err(AppError::internal(format!(
            "git push -u origin {branch} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

pub fn create_pr(
    repo: &Path,
    title: &str,
    body: &str,
    base: &str,
    draft: bool,
) -> Result<String, AppError> {
    let mut args: Vec<&str> = vec!["pr", "create", "--title", title, "--body", body, "--base", base];
    if draft {
        args.push("--draft");
    }
    let out = Command::new("gh")
        .args(&args)
        .current_dir(repo)
        .output()
        .map_err(|e| AppError::internal(format!("gh pr create: {e}")))?;
    if !out.status.success() {
        return Err(AppError::internal(format!(
            "gh pr create failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let url = stdout
        .lines()
        .find(|l| l.contains("github.com"))
        .map(|l| l.trim().to_string())
        .ok_or_else(|| AppError::internal(format!("gh pr create returned no URL: {}", stdout.trim())))?;
    Ok(url)
}

pub(super) fn parse_account(s: &str) -> Option<String> {
    // Match `... as someuser` (with or without trailing whitespace, parens, or comma).
    for line in s.lines() {
        if let Some(idx) = line.find(" as ") {
            let after = &line[idx + 4..];
            let acct: String = after
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '(' && *c != ',')
                .collect();
            if !acct.is_empty() {
                return Some(acct);
            }
        }
    }
    None
}

pub(super) fn which_in(name: &str, path_var: &OsStr) -> Option<String> {
    for dir in std::env::split_paths(path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&candidate) {
                    if meta.permissions().mode() & 0o111 != 0 {
                        return Some(candidate.to_string_lossy().into_owned());
                    }
                }
            }
            #[cfg(not(unix))]
            { return Some(candidate.to_string_lossy().into_owned()); }
        }
    }
    None
}
