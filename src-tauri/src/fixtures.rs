// Keep in sync with lib/mock-data.ts. lib/mock-data.ts is the canonical
// shape; if you change something here, mirror it there.

use crate::models::*;

pub fn projects() -> Vec<Project> {
    vec![Project {
        id: "proj-default".into(),
        name: "example-app".into(),
        path: "/Users/dev/example-app".into(),
        default_branch: "main".into(),
    }]
}

pub fn tasks_for_project(project_id: &str) -> Vec<Task> {
    if project_id != "proj-default" {
        return vec![];
    }
    vec![
        Task {
            id: "task-042".into(),
            project_id: project_id.into(),
            title: "Add magic link login while preserving JWT flow".into(),
            description: "Migrate sign-in to email magic links without breaking the existing JWT-based session handler.".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec![
                AcceptanceCriterion { id: "ac1".into(), label: "Magic link email delivered in dev".into(), satisfied: true },
                AcceptanceCriterion { id: "ac2".into(), label: "Existing JWT routes still pass".into(), satisfied: true },
                AcceptanceCriterion { id: "ac3".into(), label: "Session cookie behavior unchanged".into(), satisfied: false },
            ],
            constraints: vec!["No new external dependencies".into(), "Do not modify src/billing".into()],
            selected_engine: Some("claude-code".into()),
            status: TaskStatus::ReviewReady,
            risk: RiskLevel::Sensitive,
            branch_name: Some("aistudio/task-42-magic-link".into()),
            worktree_path: Some("/Users/dev/.aistudio/worktrees/example-app/task-042".into()),
            created_at: "2026-05-15T10:00:00Z".into(),
        },
        Task {
            id: "task-041".into(),
            project_id: project_id.into(),
            title: "Fix race in checkout cancellation".into(),
            description: "Investigate intermittent failure when a user cancels checkout mid-payment.".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec![
                AcceptanceCriterion { id: "ac1".into(), label: "Reproducer test added".into(), satisfied: false },
                AcceptanceCriterion { id: "ac2".into(), label: "No regressions in /checkout".into(), satisfied: false },
            ],
            constraints: vec!["Run full test suite".into()],
            selected_engine: Some("codex-cli".into()),
            status: TaskStatus::Running,
            risk: RiskLevel::Safe,
            branch_name: Some("aistudio/task-41-checkout-race".into()),
            worktree_path: Some("/Users/dev/.aistudio/worktrees/example-app/task-041".into()),
            created_at: "2026-05-16T14:00:00Z".into(),
        },
        Task {
            id: "task-040".into(),
            project_id: project_id.into(),
            title: "Reduce dashboard query latency".into(),
            description: "P95 is 1.2s; target 400ms.".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec![
                AcceptanceCriterion { id: "ac1".into(), label: "P95 under 400ms in load test".into(), satisfied: false },
            ],
            constraints: vec![],
            selected_engine: None,
            status: TaskStatus::Draft,
            risk: RiskLevel::Safe,
            branch_name: None,
            worktree_path: None,
            created_at: "2026-05-17T09:00:00Z".into(),
        },
        Task {
            id: "task-039".into(),
            project_id: project_id.into(),
            title: "Improve onboarding empty state".into(),
            description: "Show users a guided path on first login.".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec![],
            constraints: vec![],
            selected_engine: Some("claude-code".into()),
            status: TaskStatus::Approved,
            risk: RiskLevel::Safe,
            branch_name: Some("aistudio/task-39-onboarding".into()),
            worktree_path: Some("/Users/dev/.aistudio/worktrees/example-app/task-039".into()),
            created_at: "2026-05-14T11:00:00Z".into(),
        },
        Task {
            id: "task-038".into(),
            project_id: project_id.into(),
            title: "Refactor billing webhook handler".into(),
            description: "Split the 600-line handler into intent-scoped sub-handlers.".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec![],
            constraints: vec!["Do not change webhook public contract".into()],
            selected_engine: None,
            status: TaskStatus::ChangesRequested,
            risk: RiskLevel::Sensitive,
            branch_name: Some("aistudio/task-38-webhook-refactor".into()),
            worktree_path: None,
            created_at: "2026-05-13T15:00:00Z".into(),
        },
    ]
}

pub fn engines() -> Vec<EngineStatus> {
    vec![
        EngineStatus {
            id: "claude-code".into(),
            name: "Claude Code".into(),
            version: Some("0.43.1".into()),
            status: EngineDetectionStatus::Ready,
            binary_path: Some("/opt/homebrew/bin/claude".into()),
        },
        EngineStatus {
            id: "codex-cli".into(),
            name: "Codex CLI".into(),
            version: Some("0.125.0".into()),
            status: EngineDetectionStatus::NotAuthenticated,
            binary_path: Some("/opt/homebrew/bin/codex".into()),
        },
    ]
}

pub fn verification_for_task(task_id: &str) -> Vec<VerificationRun> {
    if task_id != "task-042" {
        return vec![];
    }
    vec![VerificationRun {
        id: "vr-001".into(),
        task_id: task_id.into(),
        started_at: "2026-05-17T12:00:00Z".into(),
        checks: vec![
            VerificationCheck { kind: "install".into(), status: VerificationStatus::Passed, duration_ms: Some(8400), log_excerpt: Some("Lockfile up to date".into()) },
            VerificationCheck { kind: "typecheck".into(), status: VerificationStatus::Passed, duration_ms: Some(3200), log_excerpt: None },
            VerificationCheck { kind: "lint".into(), status: VerificationStatus::Warning, duration_ms: Some(1100), log_excerpt: Some("2 warnings: unused import in auth.ts".into()) },
            VerificationCheck { kind: "test".into(), status: VerificationStatus::Passed, duration_ms: Some(18000), log_excerpt: Some("142 passed, 0 failed".into()) },
            VerificationCheck { kind: "build".into(), status: VerificationStatus::Failed, duration_ms: Some(22000), log_excerpt: Some("Type error in middleware.ts:88".into()) },
        ],
    }]
}
