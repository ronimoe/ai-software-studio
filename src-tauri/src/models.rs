use serde::{Serialize, Deserialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Draft,
    WorktreeCreated,
    Running,
    NeedsInput,
    VerificationRunning,
    ReviewReady,
    Approved,
    PrPrepared,
    Done,
    ChangesRequested,
    Rejected,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel { Safe, Sensitive, Dependency, Migration, Infra, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub out_of_scope: String,
    pub files_to_touch_hint: String,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub constraints: Vec<String>,
    pub selected_engine: Option<String>,
    pub status: TaskStatus,
    pub risk: RiskLevel,
    pub branch_name: Option<String>,
    pub worktree_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceCriterion {
    pub id: String,
    pub label: String,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EngineDetectionStatus { NotInstalled, Detected, Ready, NotAuthenticated, Error }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub status: EngineDetectionStatus,
    pub binary_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VerificationStatus { NotRun, Running, Passed, Failed, Skipped, Warning }

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCheck {
    pub kind: String,
    pub status: VerificationStatus,
    pub duration_ms: Option<u32>,
    pub log_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRun {
    pub id: String,
    pub task_id: String,
    pub started_at: String,
    pub checks: Vec<VerificationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub out_of_scope: String,
    pub files_to_touch_hint: String,
    pub acceptance_criteria: Vec<String>, // raw labels; ids assigned server-side
    pub constraints: Vec<String>,
    pub selected_engine: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: String,
    pub status: ChangeStatus,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSettings {
    pub install: Option<String>,
    pub typecheck: Option<String>,
    pub lint: Option<String>,
    pub test: Option<String>,
    pub build: Option<String>,
}

impl Default for VerificationSettings {
    fn default() -> Self {
        Self {
            install: Some("pnpm install".to_string()),
            typecheck: Some("pnpm typecheck".to_string()),
            lint: Some("pnpm lint".to_string()),
            test: Some("pnpm test".to_string()),
            build: Some("pnpm build".to_string()),
        }
    }
}
