use super::runner::run_single;
use crate::models::VerificationStatus;
use tempfile::TempDir;

#[tokio::test]
async fn run_single_passes_for_zero_exit() {
    let dir = TempDir::new().expect("tempdir");
    let r = run_single(dir.path(), "test", "true").await.expect("run");
    assert_eq!(r.kind, "test");
    assert!(matches!(r.status, VerificationStatus::Passed));
    assert!(r.duration_ms.is_some());
}

#[tokio::test]
async fn run_single_fails_for_nonzero_exit() {
    let dir = TempDir::new().expect("tempdir");
    let r = run_single(dir.path(), "test", "false").await.expect("run");
    assert!(matches!(r.status, VerificationStatus::Failed));
}

#[tokio::test]
async fn run_single_captures_log_excerpt() {
    let dir = TempDir::new().expect("tempdir");
    let r = run_single(dir.path(), "test", "echo hello && echo world && false")
        .await
        .expect("run");
    let excerpt = r.log_excerpt.expect("excerpt present");
    assert!(excerpt.contains("hello"));
    assert!(excerpt.contains("world"));
}

#[tokio::test]
async fn run_single_excerpt_is_tail_bounded_to_4kb() {
    let dir = TempDir::new().expect("tempdir");
    // Emit ~8KB; excerpt should keep the tail (~4KB).
    let r = run_single(dir.path(), "test", "yes | head -c 8000; echo END")
        .await
        .expect("run");
    let excerpt = r.log_excerpt.expect("excerpt present");
    assert!(
        excerpt.len() <= 4200,
        "excerpt should be roughly capped at 4KB, got {}",
        excerpt.len()
    );
    assert!(
        excerpt.ends_with("END\n") || excerpt.ends_with("END"),
        "should keep the tail (END marker)"
    );
}

#[tokio::test]
async fn run_single_returns_skipped_for_empty_command() {
    let dir = TempDir::new().expect("tempdir");
    let r = run_single(dir.path(), "lint", "").await.expect("run");
    assert!(matches!(r.status, VerificationStatus::Skipped));
}
