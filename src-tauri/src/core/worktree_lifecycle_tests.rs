use super::worktree_context::WorktreeContextService;
use super::worktree_lifecycle::create_worktree_lifecycle;
use crate::db::Db;
use crate::git::{worktree_paths::branch_name, GitService};
use crate::models::{CreateTaskRequest, Project, TaskStatus};
use crate::projects::ProjectService;
use crate::tasks::TaskService;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn lifecycle_does_not_leak_state_when_worktree_add_fails() {
    // In-memory DB with migrations applied (existing Db::test_pool from Plan 2).
    let db = Db::test_pool().await.expect("db");
    let tasks = TaskService::new(db.clone());
    let projects = ProjectService::new(db.clone());

    // Real git repo on disk so the project is well-formed.
    let repo_dir = TempDir::new().expect("repo tmp");
    for args in [
        &["init", "--initial-branch=main"][..],
        &["config", "user.email", "t@example.com"][..],
        &["config", "user.name", "Test"][..],
    ] {
        Command::new("git").args(args).current_dir(repo_dir.path()).status().expect("git setup");
    }
    std::fs::write(repo_dir.path().join("README.md"), "x").unwrap();
    Command::new("git").args(["add", "."]).current_dir(repo_dir.path()).status().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(repo_dir.path()).status().unwrap();

    // Insert a project + task using the existing services.
    let project = Project {
        id: format!("proj-{}", Uuid::new_v4()),
        name: "test project".into(),
        path: repo_dir.path().to_string_lossy().into_owned(),
        default_branch: "main".into(),
    };
    projects.insert(&project).await.expect("insert project");

    let task = tasks
        .create(&CreateTaskRequest {
            project_id: project.id.clone(),
            title: "test task".into(),
            description: "x".into(),
            out_of_scope: String::new(),
            files_to_touch_hint: String::new(),
            acceptance_criteria: vec![],
            constraints: vec![],
            selected_engine: None,
        })
        .await
        .expect("create task");

    let git = GitService::new();
    let wt_ctx = WorktreeContextService::new();
    let branch = branch_name(&task.id);

    // Force step 1 to fail by pre-creating the dest path as a regular file:
    // `git worktree add` will refuse to write there.
    let dest_parent = TempDir::new().expect("dest parent");
    let dest = dest_parent.path().join("wt");
    std::fs::write(&dest, "blocker").unwrap();

    let result = create_worktree_lifecycle(
        &git, &wt_ctx, &tasks, &task,
        repo_dir.path(), &branch, &dest,
        &project.default_branch,
    ).await;

    assert!(result.is_err(), "lifecycle must fail when worktree_add cannot proceed");

    // Compensating-action contract: no leaked DB state. Task is still Draft,
    // no branch/worktree_path persisted.
    let reloaded = tasks.get(&task.id).await.expect("task still exists");
    assert_eq!(reloaded.status, TaskStatus::Draft, "task status unchanged");
    assert!(reloaded.branch_name.is_none(), "no branch recorded");
    assert!(reloaded.worktree_path.is_none(), "no worktree path recorded");

    // Step-1 failure means `git worktree add` itself failed before ever creating
    // the branch ref, so there should be nothing to clean up — but verify the
    // user's repo is clean regardless, so the user can immediately retry.
    let branches = Command::new("git")
        .args(["branch", "--list", &branch])
        .current_dir(repo_dir.path())
        .output()
        .expect("list branches");
    assert!(
        String::from_utf8_lossy(&branches.stdout).trim().is_empty(),
        "no dangling branch ref must be left in the user's repo after step-1 failure"
    );
}
