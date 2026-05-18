use super::repository::ProjectRepository;
use crate::db::Db;
use crate::models::Project;

async fn fresh_repo() -> (Db, ProjectRepository) {
    let db = Db::test_pool().await.expect("test pool");
    let repo = ProjectRepository::new(db.clone());
    (db, repo)
}

fn sample(path: &str) -> Project {
    Project {
        id: "proj-test-1".to_string(),
        name: "example".to_string(),
        path: path.to_string(),
        default_branch: "main".to_string(),
    }
}

#[tokio::test]
async fn insert_then_list_returns_the_project() {
    let (_db, repo) = fresh_repo().await;
    repo.insert(&sample("/tmp/example")).await.expect("insert");
    let listed = repo.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].path, "/tmp/example");
    assert_eq!(listed[0].name, "example");
}

#[tokio::test]
async fn insert_duplicate_path_returns_error() {
    let (_db, repo) = fresh_repo().await;
    repo.insert(&sample("/tmp/dup")).await.expect("first insert");
    let mut second = sample("/tmp/dup");
    second.id = "proj-test-2".into();
    let err = repo.insert(&second).await.expect_err("duplicate should fail");
    assert!(err.message.to_lowercase().contains("unique") || err.message.to_lowercase().contains("constraint"),
        "expected unique-constraint error, got: {}", err.message);
}

#[tokio::test]
async fn get_returns_project_by_id() {
    let (_db, repo) = fresh_repo().await;
    repo.insert(&sample("/tmp/get")).await.expect("insert");
    let got = repo.get("proj-test-1").await.expect("get");
    assert_eq!(got.path, "/tmp/get");
}

#[tokio::test]
async fn get_unknown_id_returns_not_found() {
    let (_db, repo) = fresh_repo().await;
    let err = repo.get("nope").await.expect_err("should not find");
    assert!(matches!(err.code, crate::error::AppErrorCode::NotFound));
}

#[tokio::test]
async fn list_returns_empty_when_no_projects() {
    let (_db, repo) = fresh_repo().await;
    let listed = repo.list().await.expect("list");
    assert!(listed.is_empty());
}
