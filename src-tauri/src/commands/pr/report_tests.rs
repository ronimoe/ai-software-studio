use super::report::render;
use crate::models::{
    AcceptanceCriterion, ChangeStatus, ChangedFile, RiskLevel, Task, TaskStatus,
    VerificationCheck, VerificationRun, VerificationStatus,
};

fn sample_task() -> Task {
    Task {
        id: "task-x".into(),
        project_id: "proj-x".into(),
        title: "Add magic link".into(),
        description: "Wire magic links.".into(),
        out_of_scope: "".into(),
        files_to_touch_hint: "".into(),
        acceptance_criteria: vec![AcceptanceCriterion {
            id: "ac1".into(), label: "Email delivered".into(), satisfied: true,
        }],
        constraints: vec!["no new deps".into()],
        selected_engine: None,
        status: TaskStatus::ReviewReady,
        risk: RiskLevel::Unknown,
        branch_name: Some("aistudio/task-x".into()),
        worktree_path: Some("/tmp/wt".into()),
        created_at: "2026-05-18".into(),
    }
}

#[test]
fn renders_h1_title_section_header() {
    let r = render(&sample_task(), &[], None);
    assert!(r.starts_with("# AI Software Studio Evidence Report"));
    assert!(r.contains("## Task"));
    assert!(r.contains("Add magic link"));
}

#[test]
fn renders_files_changed_table_with_rows() {
    let files = vec![
        ChangedFile { path: "src/a.ts".into(), status: ChangeStatus::Modified, additions: 0, deletions: 0 },
        ChangedFile { path: "src/b.ts".into(), status: ChangeStatus::Added, additions: 0, deletions: 0 },
    ];
    let r = render(&sample_task(), &files, None);
    assert!(r.contains("| File | Status |"));
    assert!(r.contains("| `src/a.ts` | Modified |"));
    assert!(r.contains("| `src/b.ts` | Added |"));
}

#[test]
fn renders_verification_table_when_run_present() {
    let run = VerificationRun {
        id: "vr-1".into(),
        task_id: "task-x".into(),
        started_at: "2026-05-18".into(),
        checks: vec![
            VerificationCheck { kind: "test".into(), status: VerificationStatus::Passed, duration_ms: Some(100), log_excerpt: None },
            VerificationCheck { kind: "build".into(), status: VerificationStatus::Failed, duration_ms: Some(200), log_excerpt: None },
        ],
    };
    let r = render(&sample_task(), &[], Some(&run));
    assert!(r.contains("✅ passed"));
    assert!(r.contains("❌ failed"));
    assert!(r.contains("| test |"));
}

#[test]
fn renders_constraints_section_when_present() {
    let r = render(&sample_task(), &[], None);
    assert!(r.contains("## Constraints"));
    assert!(r.contains("- no new deps"));
}

#[test]
fn renders_satisfied_criteria_as_checked() {
    let r = render(&sample_task(), &[], None);
    assert!(r.contains("- [x] Email delivered"));
}

#[test]
fn no_changes_section_says_no_changed_files() {
    let r = render(&sample_task(), &[], None);
    assert!(r.contains("_No changed files detected._"));
}
