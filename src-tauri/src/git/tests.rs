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
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .status()
        .expect("add");
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
    let dest = TempDir::new().expect("dest tempdir").keep().join("wt");
    let result = svc
        .worktree_add(repo.path(), "aistudio/test-branch", &dest, None)
        .expect("add");
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
    let dest = TempDir::new().expect("dest").keep().join("wt");
    let err = svc
        .worktree_add(not_repo.path(), "aistudio/x", &dest, None)
        .expect_err("should fail");
    assert!(err.message.to_lowercase().contains("git"));
}

#[test]
fn worktree_remove_cleans_up() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").keep().join("wt");
    svc.worktree_add(repo.path(), "aistudio/cleanup", &dest, None)
        .expect("add");
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
    let dest = TempDir::new().expect("dest").keep().join("wt");
    svc.worktree_add(repo.path(), "aistudio/idem", &dest, None)
        .expect("add");
    svc.worktree_remove(repo.path(), &dest)
        .expect("first remove");
    // Second call on an already-removed path must succeed.
    svc.worktree_remove(repo.path(), &dest)
        .expect("second remove must be a no-op");
    assert!(!dest.exists());
}

/// `branch_delete` must be idempotent — calling it on a non-existent branch
/// must not error. Required so the rollback path can call it after the worktree
/// has been removed, regardless of whether the branch was actually created.
#[test]
fn branch_delete_is_idempotent() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").keep().join("wt");
    svc.worktree_add(repo.path(), "aistudio/branch-del", &dest, None)
        .expect("add");
    // Remove worktree first so the branch is no longer checked out anywhere.
    svc.worktree_remove(repo.path(), &dest)
        .expect("remove worktree");
    svc.branch_delete(repo.path(), "aistudio/branch-del")
        .expect("first delete");
    // Second call on a now-missing branch must succeed.
    svc.branch_delete(repo.path(), "aistudio/branch-del")
        .expect("second delete must be a no-op");
}

/// Verifies rollback contract: after worktree_remove + branch_delete the user's
/// repo is fully clean — no dangling worktree dir, no dangling branch ref.
/// Without this, a second create attempt for the same task would fail with
/// "fatal: a branch named ... already exists".
#[test]
fn worktree_remove_does_not_leak_branch_when_rollback_calls_branch_delete_separately() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").keep().join("wt");
    let branch = "aistudio/no-leak";
    svc.worktree_add(repo.path(), branch, &dest, None)
        .expect("add");
    svc.worktree_remove(repo.path(), &dest)
        .expect("remove worktree");
    svc.branch_delete(repo.path(), branch)
        .expect("delete branch");

    let branches = Command::new("git")
        .args(["branch", "--list", branch])
        .current_dir(repo.path())
        .output()
        .expect("list branches");
    assert!(
        String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
        "branch ref must be gone after rollback"
    );
}

/// Backward-compat: existing call sites use `None` for base_ref and expect the
/// new worktree to branch off the current HEAD of the parent repo.
#[test]
fn worktree_add_without_base_ref_uses_head() {
    let repo = init_repo();
    let svc = GitService::new();
    let dest = TempDir::new().expect("dest").keep().join("wt");
    svc.worktree_add(repo.path(), "aistudio/head-base", &dest, None)
        .expect("add");

    let head_repo = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("rev-parse repo HEAD");
    let head_wt = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&dest)
        .output()
        .expect("rev-parse wt HEAD");
    assert_eq!(
        String::from_utf8_lossy(&head_repo.stdout).trim(),
        String::from_utf8_lossy(&head_wt.stdout).trim(),
        "worktree HEAD matches repo HEAD when no base_ref is provided"
    );
}

/// When a base_ref is provided, the new worktree must branch off THAT ref —
/// not the parent repo's current HEAD. This is the fix for the "checked out a
/// feature branch and started a task" silent wrong-base bug.
#[test]
fn worktree_add_with_base_ref_uses_specified_ref() {
    let repo = init_repo();
    let svc = GitService::new();

    // Capture main's commit before we move off it.
    let main_sha_out = Command::new("git")
        .args(["rev-parse", "main"])
        .current_dir(repo.path())
        .output()
        .expect("rev-parse main");
    let main_sha = String::from_utf8_lossy(&main_sha_out.stdout)
        .trim()
        .to_string();

    // Create + check out feature-x with a distinct commit so HEAD != main.
    Command::new("git")
        .args(["checkout", "-b", "feature-x"])
        .current_dir(repo.path())
        .status()
        .expect("checkout feature-x");
    std::fs::write(repo.path().join("feature.txt"), "feature").expect("write feature");
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo.path())
        .status()
        .expect("add");
    Command::new("git")
        .args(["commit", "-m", "feature commit"])
        .current_dir(repo.path())
        .status()
        .expect("commit feature");

    let feature_sha_out = Command::new("git")
        .args(["rev-parse", "feature-x"])
        .current_dir(repo.path())
        .output()
        .expect("rev-parse feature-x");
    let feature_sha = String::from_utf8_lossy(&feature_sha_out.stdout)
        .trim()
        .to_string();
    assert_ne!(
        main_sha, feature_sha,
        "feature-x must be distinct from main"
    );

    let dest = TempDir::new().expect("dest").keep().join("wt");
    svc.worktree_add(repo.path(), "aistudio/with-base", &dest, Some("main"))
        .expect("add with base ref");

    let wt_head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&dest)
        .output()
        .expect("rev-parse wt HEAD");
    let wt_sha = String::from_utf8_lossy(&wt_head.stdout).trim().to_string();
    assert_eq!(
        wt_sha, main_sha,
        "worktree must branch off main, not feature-x"
    );
    assert_ne!(wt_sha, feature_sha, "worktree must NOT be on feature-x");
}
