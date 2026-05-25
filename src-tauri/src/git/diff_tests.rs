use super::diff::diff;
use std::process::Command;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    Command::new("git").args(["init", "--initial-branch=main"]).current_dir(dir.path()).status().expect("init");
    Command::new("git").args(["config", "user.email", "t@e.com"]).current_dir(dir.path()).status().expect("e");
    Command::new("git").args(["config", "user.name", "T"]).current_dir(dir.path()).status().expect("n");
    std::fs::write(dir.path().join("a.txt"), "one\ntwo\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir.path()).status().expect("add");
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir.path()).status().expect("commit");
    dir
}

#[test]
fn diff_returns_unified_patch_for_modified_file() {
    let repo = init_repo();
    std::fs::write(repo.path().join("a.txt"), "one\nthree\n").unwrap();
    let out = diff(repo.path(), "a.txt").expect("diff");
    assert!(out.contains("--- a/a.txt"));
    assert!(out.contains("+++ b/a.txt"));
    assert!(out.contains("-two"));
    assert!(out.contains("+three"));
}

#[test]
fn diff_includes_untracked_files_via_intent_to_add() {
    let repo = init_repo();
    std::fs::write(repo.path().join("new.txt"), "fresh\n").unwrap();
    let out = diff(repo.path(), "new.txt").expect("diff");
    assert!(out.contains("new.txt"));
    assert!(out.contains("+fresh"));
}

#[test]
fn diff_empty_for_unchanged_file() {
    let repo = init_repo();
    let out = diff(repo.path(), "a.txt").expect("diff");
    assert!(out.is_empty(), "unchanged file should produce empty diff, got: {out}");
}
