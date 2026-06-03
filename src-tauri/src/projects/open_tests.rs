use super::open::open_project;
use crate::db::Db;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn make_git_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir.path())
        .status()
        .expect("git init");
    dir
}

// macOS's TempDir returns /var/folders/... but `git rev-parse --show-toplevel`
// canonicalizes through the /var → /private/var symlink. Use the suffix to compare.
fn assert_paths_equivalent(actual: &str, expected: &std::path::Path) {
    let expected_str = expected.to_string_lossy();
    assert!(
        actual.ends_with(expected_str.as_ref()) || expected_str.ends_with(actual),
        "expected path equivalent to {expected_str}, got {actual}"
    );
}

#[tokio::test]
async fn open_project_inserts_a_git_repo() {
    let db = Db::test_pool().await.expect("db");
    let repo = make_git_repo();
    let project = open_project(&db, repo.path().to_str().unwrap())
        .await
        .expect("open");
    assert_paths_equivalent(&project.path, repo.path());
    assert!(!project.id.is_empty());
    assert_eq!(project.default_branch, "main");
}

#[tokio::test]
async fn open_project_canonicalizes_subdirectory_to_repo_root() {
    // Regression test: selecting a subdirectory of a git repo should resolve to
    // the repo root, not store the subdirectory path. (Caught by codex review.)
    let db = Db::test_pool().await.expect("db");
    let repo = make_git_repo();
    let subdir = repo.path().join("src");
    fs::create_dir(&subdir).expect("mkdir src");

    let from_subdir = open_project(&db, subdir.to_str().unwrap())
        .await
        .expect("open from subdir");
    let from_root = open_project(&db, repo.path().to_str().unwrap())
        .await
        .expect("open from root");

    assert_paths_equivalent(&from_subdir.path, repo.path());
    assert_eq!(
        from_subdir.id, from_root.id,
        "subdirectory selection should resolve to same project as root selection"
    );
}

#[tokio::test]
async fn open_project_rejects_non_git_directory() {
    let db = Db::test_pool().await.expect("db");
    let dir = TempDir::new().expect("tempdir");
    let err = open_project(&db, dir.path().to_str().unwrap())
        .await
        .expect_err("should reject");
    assert!(
        err.message.to_lowercase().contains("git"),
        "error should mention git, got: {}",
        err.message
    );
}

#[tokio::test]
async fn open_project_rejects_nonexistent_path() {
    let db = Db::test_pool().await.expect("db");
    let err = open_project(&db, "/this/path/does/not/exist/at/all")
        .await
        .expect_err("should reject");
    assert!(
        err.message.to_lowercase().contains("exist")
            || err.message.to_lowercase().contains("not found")
    );
}

#[tokio::test]
async fn open_project_is_idempotent_on_same_path() {
    let db = Db::test_pool().await.expect("db");
    let repo = make_git_repo();
    let p1 = open_project(&db, repo.path().to_str().unwrap())
        .await
        .expect("first");
    let p2 = open_project(&db, repo.path().to_str().unwrap())
        .await
        .expect("second");
    assert_eq!(
        p1.id, p2.id,
        "re-opening the same path should return the same project"
    );
}
