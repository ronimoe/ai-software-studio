use super::*;
use std::process::Command;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir.path())
        .status()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir.path())
        .status()
        .expect("config email");
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .status()
        .expect("config name");
    std::fs::write(dir.path().join("README.md"), "hello").expect("write");
    Command::new("git").args(["add", "."]).current_dir(dir.path()).status().expect("add");
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(dir.path())
        .status()
        .expect("commit");
    dir
}

#[test]
fn worktree_add_creates_directory_and_branch() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest tempdir").into_path().join("wt");
    let result = svc.worktree_add(repo.path(), "aistudio/test-branch", &dest).expect("add");
    assert_eq!(result, dest, "returns the worktree path");
    assert!(dest.exists(), "worktree dir exists");
    assert!(dest.join(".git").exists(), "worktree has .git file");

    let branches = Command::new("git")
        .args(["branch", "--list"])
        .current_dir(repo.path())
        .output()
        .expect("list branches");
    assert!(
        String::from_utf8_lossy(&branches.stdout).contains("aistudio/test-branch"),
        "branch created"
    );
}

#[test]
fn worktree_add_rejects_when_repo_is_not_git() {
    let not_repo = TempDir::new().expect("tempdir");
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").into_path().join("wt");
    let err = svc
        .worktree_add(not_repo.path(), "aistudio/x", &dest)
        .expect_err("should fail");
    assert!(err.message.to_lowercase().contains("git"));
}

#[test]
fn worktree_remove_cleans_up() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").into_path().join("wt");
    svc.worktree_add(repo.path(), "aistudio/cleanup", &dest).expect("add");
    assert!(dest.exists());
    svc.worktree_remove(repo.path(), &dest).expect("remove");
    assert!(!dest.exists(), "worktree dir removed");
}

/// Compensating-action cleanup may invoke `worktree_remove` on a path that
/// is already gone (e.g., rollback after a partial failure where the previous
/// cleanup also fired). This must not error — see Architecture §Failure semantics.
#[test]
fn worktree_remove_is_idempotent() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").into_path().join("wt");
    svc.worktree_add(repo.path(), "aistudio/idem", &dest).expect("add");
    svc.worktree_remove(repo.path(), &dest).expect("first remove");
    // Second call on an already-removed path must succeed.
    svc.worktree_remove(repo.path(), &dest).expect("second remove must be a no-op");
    assert!(!dest.exists());
}
