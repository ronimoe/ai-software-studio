use super::worktree_context::{WorktreeContextService, MANAGED_BEGIN, MANAGED_END};
use crate::models::{AcceptanceCriterion, RiskLevel, Task, TaskStatus};
use tempfile::TempDir;

fn sample_task() -> Task {
    Task {
        id: "task-abc".into(),
        project_id: "proj-x".into(),
        title: "T".into(),
        description: "D".into(),
        out_of_scope: "".into(),
        files_to_touch_hint: "".into(),
        acceptance_criteria: vec![AcceptanceCriterion {
            id: "ac1".into(),
            label: "do thing".into(),
            satisfied: false,
        }],
        constraints: vec!["no deps".into()],
        selected_engine: None,
        status: TaskStatus::Draft,
        risk: RiskLevel::Unknown,
        branch_name: None,
        worktree_path: None,
        created_at: "2026-05-18".into(),
        queued_at: None,
    }
}

#[test]
fn install_creates_aistudio_dir_with_task_brief() {
    let wt = TempDir::new().expect("wt");
    let svc = WorktreeContextService::new();
    svc.install(wt.path(), &sample_task(), "# brief").expect("install");
    let brief = wt.path().join(".aistudio/task-brief.md");
    assert!(brief.exists(), "brief written into worktree");
    let body = std::fs::read_to_string(&brief).unwrap();
    assert!(body.contains("# brief"));
}

#[test]
fn install_creates_claude_md_with_import_reference() {
    let wt = TempDir::new().expect("wt");
    let svc = WorktreeContextService::new();
    svc.install(wt.path(), &sample_task(), "# brief").expect("install");
    let body = std::fs::read_to_string(wt.path().join("CLAUDE.md")).unwrap();
    assert!(body.contains(MANAGED_BEGIN), "managed begin marker");
    assert!(body.contains(MANAGED_END), "managed end marker");
    assert!(
        body.contains("@.aistudio/task-brief.md"),
        "imports the brief via @-syntax"
    );
}

#[test]
fn install_preserves_existing_claude_md_content() {
    let wt = TempDir::new().expect("wt");
    std::fs::write(wt.path().join("CLAUDE.md"), "# Project CLAUDE\n\nProject-level instructions.\n").unwrap();
    let svc = WorktreeContextService::new();
    svc.install(wt.path(), &sample_task(), "# brief").expect("install");
    let body = std::fs::read_to_string(wt.path().join("CLAUDE.md")).unwrap();
    assert!(body.contains("# Project CLAUDE"), "existing H1 preserved");
    assert!(body.contains("Project-level instructions."), "existing body preserved");
    assert!(body.contains(MANAGED_BEGIN));
    assert!(body.contains(MANAGED_END));
}

#[test]
fn install_replaces_existing_managed_section_on_repeat_call() {
    let wt = TempDir::new().expect("wt");
    let svc = WorktreeContextService::new();
    svc.install(wt.path(), &sample_task(), "# brief one").expect("first install");
    svc.install(wt.path(), &sample_task(), "# brief two").expect("second install");
    let body = std::fs::read_to_string(wt.path().join("CLAUDE.md")).unwrap();
    // Only one managed section.
    let begin_count = body.matches(MANAGED_BEGIN).count();
    let end_count = body.matches(MANAGED_END).count();
    assert_eq!(begin_count, 1, "exactly one managed begin marker");
    assert_eq!(end_count, 1, "exactly one managed end marker");
}

#[test]
fn install_adds_aistudio_to_gitignore() {
    let wt = TempDir::new().expect("wt");
    let svc = WorktreeContextService::new();
    svc.install(wt.path(), &sample_task(), "# brief").expect("install");
    let body = std::fs::read_to_string(wt.path().join(".gitignore")).unwrap();
    assert!(body.lines().any(|l| l.trim() == ".aistudio/" || l.trim() == ".aistudio"));
}

#[test]
fn install_does_not_duplicate_gitignore_entry() {
    let wt = TempDir::new().expect("wt");
    std::fs::write(wt.path().join(".gitignore"), "node_modules/\n.aistudio/\n").unwrap();
    let svc = WorktreeContextService::new();
    svc.install(wt.path(), &sample_task(), "# brief").expect("install");
    let body = std::fs::read_to_string(wt.path().join(".gitignore")).unwrap();
    let count = body.lines().filter(|l| l.trim() == ".aistudio/").count();
    assert_eq!(count, 1, "should not add a duplicate entry");
}

/// Install must surface a clear error when it can't write its files. This is the
/// hook the orchestrator's compensating-action cleanup tests rely on to simulate
/// a step-2 failure — see Architecture §Failure semantics.
#[test]
fn install_returns_error_when_aistudio_path_is_a_file() {
    let wt = TempDir::new().expect("wt");
    // Pre-create `.aistudio` as a FILE, not a directory. `mkdir_all` will fail.
    std::fs::write(wt.path().join(".aistudio"), "blocker").unwrap();
    let svc = WorktreeContextService::new();
    let err = svc
        .install(wt.path(), &sample_task(), "# brief")
        .expect_err("install must fail when .aistudio is a file");
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains(".aistudio") || msg.contains("mkdir"),
        "error mentions the offending path or the failed operation: {}",
        err.message
    );
}
