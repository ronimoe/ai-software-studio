use super::brief::render_brief;
use crate::models::{AcceptanceCriterion, RiskLevel, Task, TaskStatus};

fn sample_task() -> Task {
    Task {
        id: "task-abc".into(),
        project_id: "proj-x".into(),
        title: "Add magic link login".into(),
        description: "Wire magic links without breaking JWT.".into(),
        out_of_scope: "Do not touch billing".into(),
        files_to_touch_hint: "src/auth/**".into(),
        acceptance_criteria: vec![
            AcceptanceCriterion {
                id: "ac1".into(),
                label: "Email delivered in dev".into(),
                satisfied: false,
            },
            AcceptanceCriterion {
                id: "ac2".into(),
                label: "JWT routes still pass".into(),
                satisfied: false,
            },
        ],
        constraints: vec!["No new deps".into(), "Test suite green".into()],
        selected_engine: Some("claude-code".into()),
        status: TaskStatus::Draft,
        risk: RiskLevel::Unknown,
        branch_name: None,
        worktree_path: None,
        created_at: "2026-05-18T10:00:00Z".into(),
        queued_at: None,
    }
}

#[test]
fn render_includes_title_as_h1() {
    let out = render_brief(&sample_task());
    assert!(out.contains("# Add magic link login"), "missing H1");
}

#[test]
fn render_includes_description_section() {
    let out = render_brief(&sample_task());
    assert!(out.contains("## Description"));
    assert!(out.contains("Wire magic links without breaking JWT."));
}

#[test]
fn render_includes_acceptance_criteria_as_checklist() {
    let out = render_brief(&sample_task());
    assert!(out.contains("## Acceptance Criteria"));
    assert!(out.contains("- [ ] Email delivered in dev"));
    assert!(out.contains("- [ ] JWT routes still pass"));
}

#[test]
fn render_includes_satisfied_criteria_as_checked() {
    let mut t = sample_task();
    t.acceptance_criteria[0].satisfied = true;
    let out = render_brief(&t);
    assert!(out.contains("- [x] Email delivered in dev"));
}

#[test]
fn render_includes_constraints() {
    let out = render_brief(&sample_task());
    assert!(out.contains("## Constraints"));
    assert!(out.contains("- No new deps"));
    assert!(out.contains("- Test suite green"));
}

#[test]
fn render_includes_out_of_scope() {
    let out = render_brief(&sample_task());
    assert!(out.contains("## Out of Scope"));
    assert!(out.contains("Do not touch billing"));
}

#[test]
fn render_omits_empty_files_hint() {
    let mut t = sample_task();
    t.files_to_touch_hint = "".into();
    let out = render_brief(&t);
    assert!(!out.contains("## Files to Touch"));
}

#[test]
fn render_includes_files_hint_when_present() {
    let out = render_brief(&sample_task());
    assert!(out.contains("## Files to Touch"));
    assert!(out.contains("src/auth/**"));
}

#[test]
fn render_ends_with_a_trailing_newline() {
    let out = render_brief(&sample_task());
    assert!(out.ends_with("\n"), "should end with newline");
}
