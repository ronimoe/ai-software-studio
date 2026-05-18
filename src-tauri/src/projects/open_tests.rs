use super::open::open_project;
use crate::db::Db;
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

#[tokio::test]
async fn open_project_inserts_a_git_repo() {
    let db = Db::test_pool().await.expect("db");
    let repo = make_git_repo();
    let project = open_project(&db, repo.path().to_str().unwrap())
        .await
        .expect("open");
    assert_eq!(project.path, repo.path().to_string_lossy());
    assert!(!project.id.is_empty());
    assert_eq!(project.default_branch, "main");
}

#[tokio::test]
async fn open_project_rejects_non_git_directory() {
    let db = Db::test_pool().await.expect("db");
    let dir = TempDir::new().expect("tempdir");
    let err = open_project(&db, dir.path().to_str().unwrap())
        .await
        .expect_err("should reject");
    assert!(err.message.to_lowercase().contains("git"),
        "error should mention git, got: {}", err.message);
}

#[tokio::test]
async fn open_project_rejects_nonexistent_path() {
    let db = Db::test_pool().await.expect("db");
    let err = open_project(&db, "/this/path/does/not/exist/at/all")
        .await
        .expect_err("should reject");
    assert!(err.message.to_lowercase().contains("exist")
        || err.message.to_lowercase().contains("not found"));
}

#[tokio::test]
async fn open_project_is_idempotent_on_same_path() {
    let db = Db::test_pool().await.expect("db");
    let repo = make_git_repo();
    let p1 = open_project(&db, repo.path().to_str().unwrap()).await.expect("first");
    let p2 = open_project(&db, repo.path().to_str().unwrap()).await.expect("second");
    assert_eq!(p1.id, p2.id, "re-opening the same path should return the same project");
}
