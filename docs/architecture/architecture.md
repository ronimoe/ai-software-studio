# Architecture Document: AI Software Studio

**Version:** 0.1  
**Platform:** macOS + Linux only  
**Product Type:** Local-first AI coding agent command center  
**Primary Engines:** Claude Code CLI, Codex CLI  
**Primary Stack:** Tauri + Rust + Next.js + SQLite  

---

## 1. Executive Summary

AI Software Studio is a local-first desktop application that helps developers safely delegate software tasks to local AI coding agents such as Claude Code and Codex CLI.

The app does not directly call OpenAI or Anthropic APIs. Instead, it uses the developer’s already-installed local coding agents. Those agents handle their own authentication, subscription, and usage limits.

The product provides the missing workflow layer around local AI coding agents:

```text
Task
→ Constraints
→ Isolated worktree
→ Agent execution
→ Terminal/activity capture
→ Changed files
→ Verification
→ Review room
→ PR evidence report
```

---

## 2. Product Goals

## 2.1 Primary Goal

Create a local application that turns AI coding agents into a structured, reviewable, evidence-backed development workflow.

## 2.2 Technical Goal

Build a macOS/Linux desktop app that can:

- open a local Git repository
- detect local AI coding engines
- create isolated Git worktrees
- run Claude Code or Codex inside a task workspace
- stream terminal output to the UI
- capture changed files and diffs
- run verification commands
- store logs, results, and artifacts locally
- generate a review/evidence report

---

## 3. Non-Goals

The first version is not:

- a hosted SaaS
- a full cloud IDE
- a replacement for VS Code, Cursor, or terminal
- a direct OpenAI/Anthropic API wrapper
- a tool that stores user provider API tokens
- a multi-user team platform
- a Windows app
- a production deployment automation tool
- a fully autonomous agent with no human approval

---

## 4. Selected Stack

| Layer | Technology |
|---|---|
| Desktop Shell | Tauri |
| Native Runtime | Rust |
| Frontend UI | Next.js + React + TypeScript |
| Local Database | SQLite |
| Rust DB Library | sqlx |
| Styling | Tailwind CSS + shadcn/ui |
| State Management | Zustand / TanStack Query |
| Diff Viewer | Monaco Editor or react-diff-view |
| Process Execution | Rust `tokio::process` |
| PTY / Terminal | Unix PTY library such as `portable-pty` |
| Git Integration | Git CLI first |
| Config Format | YAML |
| Artifact Storage | Local filesystem |
| Supported OS | macOS, Linux |

---

## 5. High-Level Architecture

```text
┌────────────────────────────────────────────────────────────┐
│                        Next.js UI                          │
│                                                            │
│  Task Board  |  Agent Workspace  |  Review Room            │
│  Settings    |  Verification     |  Diff Viewer            │
└─────────────────────────────┬──────────────────────────────┘
                              │
                              │ Tauri Command Bridge
                              ↓
┌────────────────────────────────────────────────────────────┐
│                      Rust Native Core                      │
│                                                            │
│  Task Service        Engine Service       Git Service       │
│  Project Service     Process Runner       Verification      │
│  Policy Engine       Artifact Store       SQLite Access     │
└─────────────────────────────┬──────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ↓                   ↓                   ↓
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   Claude Code   │  │    Codex CLI     │  │     Git CLI      │
│   local engine  │  │   local engine   │  │  worktree/diff   │
└─────────────────┘  └─────────────────┘  └─────────────────┘
                              │
                              ↓
┌────────────────────────────────────────────────────────────┐
│                  Local Filesystem + SQLite                 │
│                                                            │
│  app.db                                                    │
│  worktrees/                                                │
│  artifacts/                                                │
│  logs/                                                     │
│  reports/                                                  │
└────────────────────────────────────────────────────────────┘
```

---

## 6. Core Architecture Principle

The system is divided into two major parts:

## 6.1 Frontend UI

The Next.js UI is responsible for:

- rendering screens
- capturing user input
- displaying tasks, logs, diffs, and reports
- showing verification results
- sending user actions to Rust through Tauri commands

The frontend must not directly run shell commands.

## 6.2 Native Runtime

The Rust native core is responsible for:

- running local commands
- starting and stopping agent processes
- managing Git worktrees
- reading and writing files
- storing local data
- capturing logs
- running verification checks
- enforcing safety rules
- creating artifacts

This keeps dangerous local operations out of the UI layer.

---

## 7. Runtime Flow

```text
1. User opens project.
2. App validates Git repository.
3. App detects local engines.
4. User creates task.
5. User adds acceptance criteria and constraints.
6. App creates Git worktree.
7. User selects Claude Code or Codex.
8. App generates structured task prompt.
9. Rust core starts engine process inside worktree.
10. Engine output streams to UI.
11. App tracks changed files.
12. User runs verification.
13. App captures test/lint/build results.
14. Review room displays summary, diff, risks, and evidence.
15. User approves, requests changes, or rejects.
16. App generates PR evidence report.
```

---

## 8. Application Modules

## 8.1 Frontend Modules

```text
frontend/
├── app/
├── components/
│   ├── task-board/
│   ├── agent-workspace/
│   ├── command-panel/
│   ├── terminal-view/
│   ├── diff-viewer/
│   ├── verification-panel/
│   └── review-room/
├── features/
│   ├── projects/
│   ├── tasks/
│   ├── engines/
│   ├── worktrees/
│   ├── verification/
│   └── artifacts/
└── lib/
```

## 8.2 Rust Native Modules

```text
src-tauri/src/
├── main.rs
├── commands/
├── core/
├── engines/
├── git/
├── process/
├── verification/
├── policy/
├── db/
├── artifacts/
├── config/
└── platform/
```

---

## 9. Key Services

| Service | Responsibility |
|---|---|
| Project Service | Open repos, validate Git, load config |
| Engine Service | Detect engines, run agents, stream events |
| Git Service | Worktrees, status, changed files, diffs |
| Verification Service | Run tests, lint, typecheck, build |
| Policy Engine | Sensitive path detection, risk labels |
| Artifact Service | Logs, reports, diffs, evidence |
| Database Service | SQLite persistence |

---

## 10. Data Storage

Use two storage types:

| Type | Use |
|---|---|
| SQLite | Structured metadata |
| Filesystem | Large artifacts, logs, diffs, reports |

## App Data Location

macOS:

```text
~/Library/Application Support/AI Software Studio/
```

Linux:

```text
~/.local/share/ai-software-studio/
```

## App Data Structure

```text
AI Software Studio/
├── app.db
├── projects/
│   └── {project_id}/
│       └── tasks/
│           └── {task_id}/
│               ├── logs/
│               ├── diffs/
│               ├── reports/
│               └── artifacts/
└── worktrees/
    └── {project_slug}/
        └── {task_id}/
```

---

## 11. Engine Adapter Design

The engine adapter allows the app to support multiple local coding agents through a common interface.

```rust
pub trait EngineAdapter {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    async fn detect(&self) -> EngineDetectionResult;
    async fn health_check(&self) -> EngineHealthResult;
    async fn start_task(&self, input: StartTaskInput) -> Result<EngineRunHandle>;
    async fn send_message(&self, run_id: String, message: String) -> Result<()>;
    async fn stop(&self, run_id: String) -> Result<()>;
}
```

Initial engines:

```text
claude-code
codex-cli
```

---

## 12. Git Worktree Design

Each agent task runs in an isolated workspace.

Branch example:

```text
aistudio/task-12-magic-link-login
```

Worktree creation:

```bash
git worktree add {worktree_path} -b {branch_name}
```

Reasons:

- protects the main working tree
- makes review easier
- supports future parallel tasks
- creates clear task boundaries

---

## 13. Process and Terminal Design

The Rust process runner must support:

- spawning a process in a specific working directory
- streaming stdout/stderr
- writing input to stdin
- stopping the process
- force-killing the process
- recording exit code and timestamps

Because Claude Code and Codex CLI may behave interactively, the app should support Unix PTY sessions.

Stop behavior:

```text
1. Send SIGTERM.
2. Wait briefly.
3. If still running, send SIGKILL.
4. Mark run as stopped.
5. Save terminal log.
```

---

## 14. Verification Design

Verification must be independent from the agent.

The app should not trust agent claims like:

```text
Tests passed.
```

The app should run checks itself.

Verification commands come from `.aistudio/config.yaml`.

Example:

```yaml
commands:
  test: "pnpm test"
  lint: "pnpm lint"
  typecheck: "pnpm typecheck"
  build: "pnpm build"
```

---

## 15. Security Architecture

Security principles:

- no provider API tokens stored by the app
- Claude Code and Codex handle their own login
- no direct shell execution from frontend
- all local commands go through Rust bridge
- work happens inside isolated Git worktrees
- sensitive path changes are flagged
- destructive actions require confirmation
- logs and artifacts stay local by default

Dangerous actions requiring confirmation:

- deleting worktrees
- adding dependencies
- changing lockfiles
- creating migrations
- touching `.env`
- modifying auth
- modifying billing
- modifying infrastructure
- pushing branches
- creating PRs

---

## 16. Development Phases

## Phase 1: Technical Foundation

- Tauri window
- Next.js UI
- detect Git
- detect Claude Code
- detect Codex
- run simple command
- stream output to UI

## Phase 2: Project and Task Model

- open repo
- save project
- create task
- acceptance criteria
- constraints
- SQLite persistence

## Phase 3: Worktree + Engine Run

- create task branch
- create worktree
- generate prompt
- start Claude/Codex
- stream output
- save engine events

## Phase 4: Diff + Changed Files

- Git status
- changed file list
- diff viewer
- sensitive path detection

## Phase 5: Verification

- load commands from config
- run test/lint/typecheck/build
- capture logs
- show pass/fail results

## Phase 6: Review Room

- summary
- criteria checklist
- changed files
- verification results
- risk report
- approve/request changes/reject
- generate PR evidence Markdown

---

## 17. Major Technical Risks

| Risk | Mitigation |
|---|---|
| Interactive CLI behavior | Use PTY and external terminal fallback |
| Engine output parsing | Capture raw logs, use Git diff and verification as truth |
| Long-running process hangs | Stop/kill controls and saved partial logs |
| Sensitive data in logs | Local-only storage first, redaction later |
| Git worktree conflicts | Validate repo state and clear branch naming |

---

## 18. Final Architecture Decision

The selected architecture is:

```text
Tauri + Rust native core
Next.js UI
SQLite local database
Git worktree isolation
Claude Code / Codex CLI as external local engines
macOS + Linux only
```

The core technical bet:

> Build the product as a local workflow/control layer around existing AI coding agents, not as a hosted model API wrapper.
