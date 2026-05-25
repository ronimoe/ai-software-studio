use super::repository::VerificationRepository;
use super::runner::CheckResult;
use crate::db::Db;
use crate::models::VerificationStatus;

async fn seed_task(db: &Db) {
    sqlx::query("INSERT INTO projects (id, name, path, default_branch) VALUES (?, ?, ?, ?)")
        .bind("proj-1").bind("p").bind("/tmp/p").bind("main")
        .execute(&db.pool).await.expect("seed project");
    sqlx::query("INSERT INTO tasks (id, project_id, title) VALUES (?, ?, ?)")
        .bind("task-1").bind("proj-1").bind("t")
        .execute(&db.pool).await.expect("seed task");
}

#[tokio::test]
async fn insert_run_persists_checks_in_order() {
    let db = Db::test_pool().await.expect("db");
    seed_task(&db).await;
    let repo = VerificationRepository::new(db.clone());

    let checks = vec![
        CheckResult { kind: "install".into(), status: VerificationStatus::Passed, duration_ms: Some(100), log_excerpt: Some("ok".into()) },
        CheckResult { kind: "test".into(),    status: VerificationStatus::Failed, duration_ms: Some(200), log_excerpt: Some("FAIL".into()) },
    ];
    let run = repo.insert_run("task-1", checks).await.expect("insert");
    assert!(!run.id.is_empty());
    assert_eq!(run.checks.len(), 2);
    assert_eq!(run.checks[0].kind, "install");
    assert_eq!(run.checks[1].kind, "test");
    assert!(matches!(run.checks[1].status, VerificationStatus::Failed));
}

#[tokio::test]
async fn list_for_task_returns_runs_newest_first() {
    let db = Db::test_pool().await.expect("db");
    seed_task(&db).await;
    let repo = VerificationRepository::new(db);
    repo.insert_run("task-1", vec![CheckResult { kind: "test".into(), status: VerificationStatus::Passed, duration_ms: Some(10), log_excerpt: None }]).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await; // ensure datetime('now') differs
    repo.insert_run("task-1", vec![CheckResult { kind: "test".into(), status: VerificationStatus::Failed, duration_ms: Some(10), log_excerpt: None }]).await.unwrap();
    let runs = repo.list_for_task("task-1").await.expect("list");
    assert_eq!(runs.len(), 2);
    assert!(matches!(runs[0].checks[0].status, VerificationStatus::Failed), "newest run first");
}
