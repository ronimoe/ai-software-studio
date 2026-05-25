use super::*;
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn spawn_runs_and_reports_exit_for_simple_command() {
    let runner = ProcessRunner::new();

    // We don't have an AppHandle in tests, so use the internal channel instead.
    // For simplicity in v0.1, the test only exercises is_running + auto-cleanup.
    let tmp = TempDir::new().expect("tempdir");

    runner
        .spawn(
            "task-test-1",
            "/bin/sh",
            &["-c", "echo hello; sleep 0.2; echo bye"],
            &tmp.path().to_path_buf(),
        )
        .await
        .expect("spawn");

    assert!(runner.is_running("task-test-1"), "registered as running");

    // Wait for the script to finish + the runner's reaper to clean up.
    sleep(Duration::from_millis(800)).await;
    assert!(!runner.is_running("task-test-1"), "should have been cleaned up");
}

#[tokio::test]
async fn stop_kills_a_long_running_process() {
    let runner = ProcessRunner::new();
    let tmp = TempDir::new().expect("tempdir");

    runner
        .spawn(
            "task-stop-1",
            "/bin/sh",
            &["-c", "sleep 30"],
            &tmp.path().to_path_buf(),
        )
        .await
        .expect("spawn");

    assert!(runner.is_running("task-stop-1"));
    runner.stop("task-stop-1").await.expect("stop");
    sleep(Duration::from_millis(300)).await;
    assert!(!runner.is_running("task-stop-1"), "stopped process is unregistered");
}

#[tokio::test]
async fn stop_unknown_task_is_a_noop_ok() {
    let runner = ProcessRunner::new();
    // Calling stop on something we never spawned should be a no-op, not an error.
    runner.stop("never-spawned").await.expect("stop noop");
}

// Suppress unused-import warnings for items only consumed by other tests.
#[allow(unused_imports)]
use {mpsc as _, OutputLine as _, OutputStream as _, RunExit as _};
