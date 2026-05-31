use super::repository::TaskRepository;
use crate::db::Db;
use crate::models::{CreateTaskRequest, TaskStatus};

async fn seed_project(db: &Db, project_id: &str) {
    sqlx::query("INSERT INTO projects (id, name, path, default_branch) VALUES (?, ?, ?, ?)")
        .bind(project_id)
        .bind("seed")
        .bind(format!("/tmp/{project_id}"))
        .bind("main")
        .execute(&db.pool)
        .await
        .expect("seed project");
}

fn request(project_id: &str, title: &str) -> CreateTaskRequest {
    CreateTaskRequest {
        project_id: project_id.into(),
        title: title.into(),
        description: "desc".into(),
        out_of_scope: "no infra".into(),
        files_to_touch_hint: "src/auth/**".into(),
        acceptance_criteria: vec!["ac one".into(), "ac two".into()],
        constraints: vec!["no new deps".into()],
        selected_engine: Some("claude-code".into()),
    }
}

#[tokio::test]
async fn insert_returns_task_with_normalized_relations() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-1").await;
    let repo = TaskRepository::new(db.clone());
    let task = repo.insert(&request("proj-1", "t1")).await.expect("insert");
    assert!(!task.id.is_empty());
    assert_eq!(task.title, "t1");
    assert_eq!(task.acceptance_criteria.len(), 2);
    assert_eq!(task.acceptance_criteria[0].label, "ac one");
    assert_eq!(task.constraints, vec!["no new deps"]);
    assert_eq!(task.status, TaskStatus::Draft);
}

#[tokio::test]
async fn list_for_project_returns_tasks_in_reverse_chronological_order() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-1").await;
    let repo = TaskRepository::new(db);
    repo.insert(&request("proj-1", "first")).await.expect("first");
    repo.insert(&request("proj-1", "second")).await.expect("second");
    let list = repo.list_for_project("proj-1").await.expect("list");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].title, "second");
    assert_eq!(list[1].title, "first");
}

#[tokio::test]
async fn list_for_project_isolates_per_project() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-a").await;
    seed_project(&db, "proj-b").await;
    let repo = TaskRepository::new(db);
    repo.insert(&request("proj-a", "a-task")).await.expect("a");
    repo.insert(&request("proj-b", "b-task")).await.expect("b");
    let a_list = repo.list_for_project("proj-a").await.expect("list a");
    let b_list = repo.list_for_project("proj-b").await.expect("list b");
    assert_eq!(a_list.len(), 1);
    assert_eq!(b_list.len(), 1);
    assert_eq!(a_list[0].title, "a-task");
    assert_eq!(b_list[0].title, "b-task");
}

#[tokio::test]
async fn get_returns_hydrated_task_with_relations() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-1").await;
    let repo = TaskRepository::new(db);
    let inserted = repo.insert(&request("proj-1", "t1")).await.expect("insert");
    let got = repo.get(&inserted.id).await.expect("get");
    assert_eq!(got.title, "t1");
    assert_eq!(got.acceptance_criteria.len(), 2);
    assert_eq!(got.constraints.len(), 1);
}

#[tokio::test]
async fn get_unknown_returns_not_found() {
    let db = Db::test_pool().await.expect("db");
    let repo = TaskRepository::new(db);
    let err = repo.get("nope").await.expect_err("should not find");
    assert!(matches!(err.code, crate::error::AppErrorCode::NotFound));
}

#[tokio::test]
async fn update_status_persists() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-1").await;
    let repo = TaskRepository::new(db);
    let t = repo.insert(&request("proj-1", "t1")).await.expect("insert");
    repo.update_status(&t.id, TaskStatus::WorktreeCreated).await.expect("update");
    let reloaded = repo.get(&t.id).await.expect("get");
    assert_eq!(reloaded.status, TaskStatus::WorktreeCreated);
}

#[tokio::test]
async fn enqueue_is_fifo_and_dequeue_resets() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-1").await;
    let repo = TaskRepository::new(db);
    let a = repo.insert(&request("proj-1", "A")).await.expect("a");
    let b = repo.insert(&request("proj-1", "B")).await.expect("b");

    repo.enqueue(&a.id).await.unwrap();
    repo.enqueue(&b.id).await.unwrap();

    // Oldest queued_at first; rowid ASC breaks ms-resolution ties deterministically.
    let next = repo.next_queued().await.unwrap().expect("a queued task");
    assert_eq!(next.id, a.id);
    assert_eq!(next.status, TaskStatus::Queued);
    assert!(next.queued_at.is_some());
    assert_eq!(repo.count_queued().await.unwrap(), 2);

    repo.dequeue(&a.id).await.unwrap();
    let after = repo.get(&a.id).await.unwrap();
    assert_eq!(after.status, TaskStatus::Draft);
    assert!(after.queued_at.is_none());
    assert_eq!(repo.count_queued().await.unwrap(), 1);
}
