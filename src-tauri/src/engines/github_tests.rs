use super::github::{parse_account, which_in};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

#[test]
fn parse_account_extracts_login_from_canonical_line() {
    let out = "github.com\n  ✓ Logged in to github.com as ronimoe (keyring)\n  ✓ Git operations protocol: ssh\n";
    assert_eq!(parse_account(out).as_deref(), Some("ronimoe"));
}

#[test]
fn parse_account_handles_trailing_paren_without_space() {
    let out = "Logged in to github.com as alice(token)";
    assert_eq!(parse_account(out).as_deref(), Some("alice"));
}

#[test]
fn parse_account_handles_trailing_comma() {
    let out = "github.com as bob, using protocol https";
    assert_eq!(parse_account(out).as_deref(), Some("bob"));
}

#[test]
fn parse_account_returns_none_when_no_login_line() {
    let out = "not a gh auth status response\nsomething else\n";
    assert!(parse_account(out).is_none());
}

#[test]
fn which_in_finds_executable_with_exec_bit() {
    let dir = TempDir::new().expect("tempdir");
    let bin = dir.path().join("gh");
    std::fs::write(&bin, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    let path: OsString = dir.path().into();
    let found = which_in("gh", &path);
    assert_eq!(found.as_deref(), bin.to_str());
}

#[test]
fn which_in_returns_none_when_file_missing() {
    let dir = TempDir::new().expect("tempdir");
    let path: OsString = dir.path().into();
    assert!(which_in("definitely-not-here", &path).is_none());
}

#[test]
fn which_in_returns_none_when_file_not_executable() {
    let dir = TempDir::new().expect("tempdir");
    let bin = dir.path().join("gh");
    std::fs::write(&bin, "not executable").unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o644)).unwrap();
    let path: OsString = dir.path().into();
    assert!(which_in("gh", &path).is_none());
}
