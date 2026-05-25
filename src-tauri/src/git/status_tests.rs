use super::status::status;
use crate::models::ChangeStatus;
use std::process::Command;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    Command::new("git").args(["init", "--initial-branch=main"]).current_dir(dir.path()).status().expect("init");
    Command::new("git").args(["config", "user.email", "t@e.com"]).current_dir(dir.path()).status().expect("email");
    Command::new("git").args(["config", "user.name", "T"]).current_dir(dir.path()).status().expect("name");
    std::fs::write(dir.path().join("a.txt"), "alpha\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir.path()).status().expect("add");
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir.path()).status().expect("commit");
    dir
}

#[test]
fn status_returns_empty_for_clean_repo() {
    let repo = init_repo();
    let out = status(repo.path()).expect("status");
    assert!(out.is_empty(), "clean repo should report no changes");
}

#[test]
fn status_reports_modified_file() {
    let repo = init_repo();
    std::fs::write(repo.path().join("a.txt"), "alpha\nbeta\n").unwrap();
    let out = status(repo.path()).expect("status");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].path, "a.txt");
    assert_eq!(out[0].status, ChangeStatus::Modified);
}

#[test]
fn status_reports_added_file() {
    let repo = init_repo();
    std::fs::write(repo.path().join("b.txt"), "bravo\n").unwrap();
    Command::new("git").args(["add", "b.txt"]).current_dir(repo.path()).status().expect("add");
    let out = status(repo.path()).expect("status");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].path, "b.txt");
    assert_eq!(out[0].status, ChangeStatus::Added);
}

#[test]
fn status_reports_deleted_file() {
    let repo = init_repo();
    std::fs::remove_file(repo.path().join("a.txt")).unwrap();
    let out = status(repo.path()).expect("status");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].path, "a.txt");
    assert_eq!(out[0].status, ChangeStatus::Deleted);
}

#[test]
fn status_reports_untracked_file() {
    let repo = init_repo();
    std::fs::write(repo.path().join("u.txt"), "untracked\n").unwrap();
    let out = status(repo.path()).expect("status");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].path, "u.txt");
    assert_eq!(out[0].status, ChangeStatus::Untracked);
}

#[test]
fn status_handles_multiple_files() {
    let repo = init_repo();
    std::fs::write(repo.path().join("a.txt"), "changed\n").unwrap();
    std::fs::write(repo.path().join("c.txt"), "new\n").unwrap();
    let mut out = status(repo.path()).expect("status");
    out.sort_by(|x, y| x.path.cmp(&y.path));
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].path, "a.txt");
    assert_eq!(out[1].path, "c.txt");
}

#[test]
fn status_errors_on_non_git_dir() {
    let dir = TempDir::new().expect("tempdir");
    let err = status(dir.path()).expect_err("should error");
    assert!(err.message.to_lowercase().contains("git"));
}
