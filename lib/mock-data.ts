import type {
  Project,
  Task,
  EngineStatus,
  VerificationRun,
} from "./bindings";
import type {
  ConversationMessage,
  ContextGraphEdge,
  ContextGraphNode,
  SnapshotMetric,
  ActiveAgent,
} from "./types";

export const mockProjects: Project[] = [
  {
    id: "proj-default",
    name: "example-app",
    path: "/Users/dev/example-app",
    defaultBranch: "main",
  },
];

export const mockTasks: Task[] = [
  {
    id: "task-042",
    projectId: "proj-default",
    title: "Add magic link login while preserving JWT flow",
    description:
      "Migrate sign-in to email magic links without breaking the existing JWT-based session handler.",
    outOfScope: "",
    filesToTouchHint: "",
    acceptanceCriteria: [
      { id: "ac1", label: "Magic link email delivered in dev", satisfied: true },
      { id: "ac2", label: "Existing JWT routes still pass", satisfied: true },
      { id: "ac3", label: "Session cookie behavior unchanged", satisfied: false },
    ],
    constraints: ["No new external dependencies", "Do not modify src/billing"],
    selectedEngine: "claude-code",
    status: "reviewReady",
    risk: "sensitive",
    branchName: "aistudio/task-42-magic-link",
    worktreePath: "/Users/dev/.aistudio/worktrees/example-app/task-042",
    createdAt: "2026-05-15T10:00:00Z",
    queuedAt: null,
  },
  {
    id: "task-041",
    projectId: "proj-default",
    title: "Fix race in checkout cancellation",
    description: "Investigate intermittent failure when a user cancels checkout mid-payment.",
    outOfScope: "",
    filesToTouchHint: "",
    acceptanceCriteria: [
      { id: "ac1", label: "Reproducer test added", satisfied: false },
      { id: "ac2", label: "No regressions in /checkout", satisfied: false },
    ],
    constraints: ["Run full test suite"],
    selectedEngine: "codex-cli",
    status: "running",
    risk: "safe",
    branchName: "aistudio/task-41-checkout-race",
    worktreePath: "/Users/dev/.aistudio/worktrees/example-app/task-041",
    createdAt: "2026-05-16T14:00:00Z",
    queuedAt: null,
  },
  {
    id: "task-040",
    projectId: "proj-default",
    title: "Reduce dashboard query latency",
    description: "P95 is 1.2s; target 400ms.",
    outOfScope: "",
    filesToTouchHint: "",
    acceptanceCriteria: [
      { id: "ac1", label: "P95 under 400ms in load test", satisfied: false },
    ],
    constraints: [],
    selectedEngine: null,
    status: "draft",
    risk: "safe",
    branchName: null,
    worktreePath: null,
    createdAt: "2026-05-17T09:00:00Z",
    queuedAt: null,
  },
  {
    id: "task-039",
    projectId: "proj-default",
    title: "Improve onboarding empty state",
    description: "Show users a guided path on first login.",
    outOfScope: "",
    filesToTouchHint: "",
    acceptanceCriteria: [],
    constraints: [],
    selectedEngine: "claude-code",
    status: "approved",
    risk: "safe",
    branchName: "aistudio/task-39-onboarding",
    worktreePath: "/Users/dev/.aistudio/worktrees/example-app/task-039",
    createdAt: "2026-05-14T11:00:00Z",
    queuedAt: null,
  },
  {
    id: "task-038",
    projectId: "proj-default",
    title: "Refactor billing webhook handler",
    description: "Split the 600-line handler into intent-scoped sub-handlers.",
    outOfScope: "",
    filesToTouchHint: "",
    acceptanceCriteria: [],
    constraints: ["Do not change webhook public contract"],
    selectedEngine: null,
    status: "changesRequested",
    risk: "sensitive",
    branchName: "aistudio/task-38-webhook-refactor",
    worktreePath: null,
    createdAt: "2026-05-13T15:00:00Z",
    queuedAt: null,
  },
];

export const mockEngines: EngineStatus[] = [
  {
    id: "claude-code",
    name: "Claude Code",
    version: "0.43.1",
    status: "ready",
    binaryPath: "/opt/homebrew/bin/claude",
  },
  {
    id: "codex-cli",
    name: "Codex CLI",
    version: "0.125.0",
    status: "notAuthenticated",
    binaryPath: "/opt/homebrew/bin/codex",
  },
];

export const mockVerification: VerificationRun[] = [
  {
    id: "vr-001",
    taskId: "task-042",
    startedAt: "2026-05-17T12:00:00Z",
    checks: [
      { kind: "install", status: "passed", durationMs: 8400, logExcerpt: "Lockfile up to date" },
      { kind: "typecheck", status: "passed", durationMs: 3200, logExcerpt: null },
      { kind: "lint", status: "warning", durationMs: 1100, logExcerpt: "2 warnings: unused import in auth.ts" },
      { kind: "test", status: "passed", durationMs: 18000, logExcerpt: "142 passed, 0 failed" },
      { kind: "build", status: "failed", durationMs: 22000, logExcerpt: "Type error in middleware.ts:88" },
    ],
  },
];

export const mockConversation: ConversationMessage[] = [
  { id: "m1", author: "user", authorName: "You", body: "Use magic link, keep JWT for now.", timestamp: "10:14" },
  { id: "m2", author: "agent", authorName: "Claude Code", body: "Acknowledged. Drafting changes in `src/auth/magic-link.ts`.", timestamp: "10:14" },
  { id: "m3", author: "agent", authorName: "Claude Code", body: "Added 14 lines, removed 3. Running tests.", timestamp: "10:18" },
  { id: "m4", author: "system", authorName: "Verification", body: "build: FAILED — type error in middleware.ts:88.", timestamp: "10:21" },
  { id: "m5", author: "user", authorName: "You", body: "Fix the middleware type — it accepts `string | undefined` now.", timestamp: "10:22" },
  { id: "m6", author: "agent", authorName: "Claude Code", body: "Fixed. Re-running build.", timestamp: "10:23" },
];

export const mockGraphNodes: ContextGraphNode[] = [
  { id: "n-task", label: "Task #042", kind: "task", x: 180, y: 80 },
  { id: "n-branch", label: "magic-link", kind: "branch", x: 60, y: 160 },
  { id: "n-claude", label: "Claude Code", kind: "engine", x: 300, y: 160 },
  { id: "n-auth", label: "src/auth/", kind: "file", x: 80, y: 240 },
  { id: "n-mid", label: "middleware.ts", kind: "file", x: 200, y: 250 },
  { id: "n-tests", label: "auth.test.ts", kind: "file", x: 320, y: 250 },
];

export const mockGraphEdges: ContextGraphEdge[] = [
  { from: "n-task", to: "n-branch" },
  { from: "n-task", to: "n-claude" },
  { from: "n-branch", to: "n-auth" },
  { from: "n-claude", to: "n-mid" },
  { from: "n-claude", to: "n-tests" },
];

export const mockSnapshot: SnapshotMetric[] = [
  { label: "Tasks in flight", value: "3", trend: "up" },
  { label: "Verification pass", value: "82%", trend: "flat" },
  { label: "Highest spend", value: "Claude Code · $4.18", trend: "up" },
];

export const mockActiveAgents: ActiveAgent[] = [
  { engineId: "claude-code", taskId: "task-042", status: "running" },
  { engineId: "codex-cli", taskId: "task-041", status: "blocked" },
];
