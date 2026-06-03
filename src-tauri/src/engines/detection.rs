use crate::{
    error::AppError,
    models::{EngineDetectionStatus, EngineStatus},
};
use std::ffi::OsStr;
use std::process::Command;

const ENGINE_ID: &str = "claude-code";
const ENGINE_NAME: &str = "Claude Code";

pub fn detect_claude() -> Result<EngineStatus, AppError> {
    detect_claude_on_path(std::env::var_os("PATH").as_deref())
}

/// Core detection with an explicit `PATH` lookup string. Kept separate from
/// [`detect_claude`] so tests can drive it without mutating the process-global
/// `PATH`, which would race other parallel tests that spawn subprocesses.
pub(crate) fn detect_claude_on_path(path: Option<&OsStr>) -> Result<EngineStatus, AppError> {
    let Some(binary) = path.and_then(|p| which_first("claude", p)) else {
        return Ok(EngineStatus {
            id: ENGINE_ID.into(),
            name: ENGINE_NAME.into(),
            version: None,
            status: EngineDetectionStatus::NotInstalled,
            binary_path: None,
        });
    };

    let version_output = Command::new(&binary)
        .arg("--version")
        .output()
        .map_err(|e| AppError::internal(format!("run claude --version: {e}")))?;

    let parsed = parse_claude_version(&String::from_utf8_lossy(&version_output.stdout));
    let status = if parsed.is_some() {
        EngineDetectionStatus::Ready
    } else {
        EngineDetectionStatus::Detected
    };

    Ok(EngineStatus {
        id: ENGINE_ID.into(),
        name: ENGINE_NAME.into(),
        version: parsed,
        status,
        binary_path: Some(binary),
    })
}

/// Return the first hit for `name` in the given `PATH` search string.
fn which_first(name: &str, path: &OsStr) -> Option<String> {
    for dir in std::env::split_paths(path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            // On Unix, ensure it's executable.
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
            {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Parse output like `"claude version 0.43.1"` or `"0.43.1 (deadbeef)"`.
/// Returns just the semver-ish prefix.
fn parse_claude_version(s: &str) -> Option<String> {
    let s = s.trim();
    // Find the first dotted-number-y token.
    for token in s.split_whitespace() {
        let stripped = token.trim_start_matches('v');
        if stripped
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && stripped.contains('.')
        {
            // Strip parenthetical suffix like "(deadbeef)".
            let clean: String = stripped
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '(' && *c != ',')
                .collect();
            if clean.split('.').count() >= 2 {
                return Some(clean);
            }
        }
    }
    None
}
