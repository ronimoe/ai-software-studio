use super::detection::detect_claude_on_path;
use crate::models::EngineDetectionStatus;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

/// Create a fake `claude` script in a fresh temp dir and return the dir so the
/// caller can build a `PATH` pointing at it. No global env mutation, so these
/// tests can't race other parallel tests that spawn subprocesses.
fn fake_claude(version_output: &str) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let bin = dir.path().join("claude");
    let script = format!("#!/bin/sh\necho '{version_output}'\n");
    std::fs::write(&bin, script).expect("write fake claude");
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    dir
}

#[test]
fn detect_claude_returns_ready_when_binary_on_path() {
    let dir = fake_claude("claude version 0.99.0");
    let path = OsString::from(dir.path());
    let s = detect_claude_on_path(Some(&path)).expect("detect");
    assert!(matches!(s.status, EngineDetectionStatus::Ready));
    assert!(s.binary_path.as_deref().unwrap_or("").ends_with("claude"));
    assert_eq!(s.version.as_deref(), Some("0.99.0"));
}

#[test]
fn detect_claude_returns_not_installed_when_missing() {
    let path = OsString::from("/nonexistent/path/only");
    let s = detect_claude_on_path(Some(&path)).expect("detect");
    assert!(matches!(s.status, EngineDetectionStatus::NotInstalled));
    assert!(s.binary_path.is_none());
    assert!(s.version.is_none());
}

#[test]
fn detect_claude_handles_unknown_version_output_gracefully() {
    let dir = fake_claude("totally unparseable garbage");
    let path = OsString::from(dir.path());
    let s = detect_claude_on_path(Some(&path)).expect("detect");
    // Binary is found, but we couldn't parse a version → Detected, not Ready.
    assert!(matches!(s.status, EngineDetectionStatus::Detected));
    assert!(s.binary_path.is_some());
    assert!(s.version.is_none());
}
