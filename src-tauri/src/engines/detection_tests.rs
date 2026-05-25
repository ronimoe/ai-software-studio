use super::detection::detect_claude;
use crate::models::EngineDetectionStatus;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;
use tempfile::TempDir;

// Serializes PATH mutations across the tests in this module so cargo's default
// parallel runner can't race them. Poison-safe via into_inner().
static PATH_LOCK: Mutex<()> = Mutex::new(());

/// Make a fake `claude` script on PATH that prints a version string, return the temp dir
/// (holding the binary) and the previous PATH so the caller can restore it.
fn with_fake_claude<F: FnOnce()>(version_output: &str, body: F) {
    let _guard = PATH_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let dir = TempDir::new().expect("tempdir");
    let bin = dir.path().join("claude");
    let script = format!("#!/bin/sh\necho '{version_output}'\n");
    std::fs::write(&bin, script).expect("write fake claude");
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).expect("chmod");

    let prev = env::var("PATH").unwrap_or_default();
    let new = format!("{}:{}", dir.path().display(), prev);
    // SAFETY for the test: this process changes PATH for the duration of the body only.
    unsafe { env::set_var("PATH", &new); }
    body();
    unsafe { env::set_var("PATH", &prev); }
}

#[test]
fn detect_claude_returns_ready_when_binary_on_path() {
    with_fake_claude("claude version 0.99.0", || {
        let s = detect_claude().expect("detect");
        assert!(matches!(s.status, EngineDetectionStatus::Ready));
        assert!(s.binary_path.as_deref().unwrap_or("").ends_with("claude"));
        assert_eq!(s.version.as_deref(), Some("0.99.0"));
    });
}

#[test]
fn detect_claude_returns_not_installed_when_missing() {
    let _guard = PATH_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let prev = env::var("PATH").unwrap_or_default();
    // SAFETY: scoped to this test; restored before return.
    unsafe { env::set_var("PATH", "/nonexistent/path/only"); }
    let s = detect_claude().expect("detect");
    assert!(matches!(s.status, EngineDetectionStatus::NotInstalled));
    assert!(s.binary_path.is_none());
    assert!(s.version.is_none());
    unsafe { env::set_var("PATH", &prev); }
}

#[test]
fn detect_claude_handles_unknown_version_output_gracefully() {
    with_fake_claude("totally unparseable garbage", || {
        let s = detect_claude().expect("detect");
        // Binary is found, but we couldn't parse a version → Detected, not Ready.
        assert!(matches!(s.status, EngineDetectionStatus::Detected));
        assert!(s.binary_path.is_some());
        assert!(s.version.is_none());
    });
}
