# Product Spec: AI Software Studio

## 1. Product Overview

**AI Software Studio** is a local-first desktop application for orchestrating AI coding agents such as **Claude Code CLI** and **Codex CLI**.

The app lets developers create structured software tasks, define acceptance criteria and constraints, run local AI agents inside isolated Git worktrees, capture output and diffs, run verification commands, and generate review-ready PR evidence reports.

The product does not directly call OpenAI or Anthropic APIs in the initial architecture. It uses locally installed agent tools that handle their own authentication and usage.

---

## 2. Product Goals

## 2.1 Primary Goal

Help developers safely delegate coding tasks to local AI agents while preserving human control, review, and approval.

## 2.2 User Goal

The user should be able to say:

```text
I assigned a scoped task to Claude Code or Codex, watched the work, verified the result, reviewed the diff, and approved or rejected the output.
```

## 2.3 Product Goal

Create a repeatable workflow:

```text
Task
→ Constraints
→ Worktree
→ Agent execution
→ Evidence
→ Verification
→ Review
→ PR report
```

---

## 3. Supported Platforms

Initial supported platforms:

- macOS
- Linux

Unsupported initially:

- Windows

Reason:

Windows adds complexity around PowerShell, paths, ConPTY, WSL, process handling, and CLI behavior differences.

---

## 4. Supported Engines

Initial engines:

- Claude Code CLI
- Codex CLI

Future engines:

- Gemini CLI
- Aider
- OpenCode
- local models
- custom enterprise agents

---

## 5. Core User Flow

```text
1. User opens AI Software Studio.
2. User selects a local Git repository.
3. App detects local engines.
4. User creates a task.
5. User defines acceptance criteria and constraints.
6. App creates an isolated Git worktree.
7. User selects Claude Code or Codex CLI.
8. App sends structured task prompt to selected engine.
9. Engine works inside the worktree.
10. App captures terminal output and events.
11. App detects changed files and Git diff.
12. User runs verification commands.
13. App stores verification output.
14. Review Room displays evidence, diff, risks, and summary.
15. User approves, requests changes, or rejects.
16. App generates PR evidence report.
```

---

## 6. Product Modules

## 6.1 Project Dashboard

Purpose:

Manage local projects.

Features:

- open local Git repo
- list recent projects
- show Git status
- detect project config
- detect available commands
- show engine readiness

## 6.2 Engine Detection

Purpose:

Show whether local engines are available.

Checks:

- `which claude`
- `which codex`
- `claude --version`
- `codex --version`

Status values:

| Status | Meaning |
|---|---|
| not_installed | CLI not found |
| detected | CLI exists |
| ready | CLI appears usable |
| not_authenticated | CLI exists but user must login |
| error | detection failed |

## 6.3 Task Board

Purpose:

Create and manage coding tasks.

Task fields:

- title
- description
- acceptance criteria
- constraints
- selected engine
- linked issue URL
- risk level
- status
- branch name
- worktree path

## 6.4 Agent Workspace

Purpose:

Run and supervise an agent for a task.

Features:

- show task brief
- show selected engine
- show worktree path
- show generated prompt
- stream terminal output
- stop agent
- request changes
- show agent events

## 6.5 Command / Conversation Panel

Purpose:

Allow structured human-agent communication.

Supported actions:

- ask
- instruct
- correct
- approve
- reject
- stop
- request changes

Important behavior:

User instructions should become persistent task constraints when appropriate.

## 6.6 Changed Files Panel

Purpose:

Show files modified by the agent.

Features:

- file path
- Git status
- additions/deletions
- risk label
- open diff
- open in editor

Risk labels:

- safe
- sensitive
- dependency
- migration
- infra
- unknown

## 6.7 Verification Dashboard

Purpose:

Run checks independently from the agent.

Checks:

- install
- test
- lint
- typecheck
- build
- custom commands

Statuses:

- not_run
- running
- passed
- failed
- skipped
- warning

## 6.8 Review Room

Purpose:

Help the human decide whether to approve or reject.

Sections:

- task summary
- acceptance criteria
- changed files
- diff
- verification results
- risk report
- human decisions
- PR evidence report

Actions:

- approve
- request changes
- reject
- generate PR report
- open worktree
- open terminal

## 6.9 Artifact Store

Purpose:

Store evidence locally.

Artifacts:

- terminal logs
- Git diffs
- verification logs
- PR reports
- review summaries
- risk reports

---

## 7. Task Status Lifecycle

```text
Draft
→ Worktree Created
→ Running
→ Needs Input
→ Verification Running
→ Review Ready
→ Approved
→ PR Prepared
→ Done
```

Alternative terminal states:

```text
Changes Requested
Rejected
Failed
Stopped
```

---

## 8. Project Config

Each repo can optionally include:

```text
.aistudio/config.yaml
```

Example:

```yaml
project:
  name: "example-app"

engines:
  default: "claude-code"
  allowed:
    - "claude-code"
    - "codex-cli"

commands:
  install: "pnpm install"
  test: "pnpm test"
  lint: "pnpm lint"
  typecheck: "pnpm typecheck"
  build: "pnpm build"

sensitive_paths:
  - "src/auth/**"
  - "src/billing/**"
  - "prisma/migrations/**"
  - ".env*"
  - "infra/**"

approval_required:
  - "database_migration"
  - "dependency_addition"
  - "auth_change"
  - "billing_change"
  - "infra_change"

pr:
  default_base_branch: "main"
```

---

## 9. Prompt Contract

Base prompt sent to local engine:

```text
You are working inside a local Git worktree for this task.

Task:
{{task_title}}

Description:
{{task_description}}

Acceptance Criteria:
{{acceptance_criteria}}

Constraints:
{{constraints}}

Sensitive Paths:
{{sensitive_paths}}

Project Commands:
- test: {{test_command}}
- lint: {{lint_command}}
- typecheck: {{typecheck_command}}
- build: {{build_command}}

Rules:
1. Do not modify files outside the requested scope unless necessary.
2. Ask before changing sensitive paths.
3. Ask before adding dependencies.
4. Ask before creating database migrations.
5. Prefer small, reviewable changes.
6. After implementation, summarize what changed.
7. Do not claim tests passed unless they were actually run.
```

---

## 10. PR Evidence Report

The app generates Markdown like:

```markdown
# AI Software Studio Evidence Report

## Task

{{task_title}}

## Summary

{{summary}}

## Acceptance Criteria

- [x] Criterion 1
- [x] Criterion 2

## Files Changed

{{changed_files}}

## Verification

{{verification_results}}

## Risks

{{risk_report}}

## Human Decisions

{{approvals}}

## Recommendation

Ready for human PR review.
```

---

## 11. MVP Requirements

## Must Have

- open local Git repo
- detect Claude Code and Codex CLI
- create task
- define acceptance criteria
- define constraints
- create Git worktree
- start engine run
- stream terminal output
- stop engine run
- show changed files
- show Git diff
- run verification commands
- store logs and results
- show review room
- generate PR evidence report

## Should Have

- sensitive path detection
- basic risk score
- open worktree in editor
- request changes loop
- copy PR report to clipboard
- project config detection

## Not MVP

- Windows support
- cloud sync
- hosted agent execution
- provider API integration
- multi-user collaboration
- automatic deployment
- advanced analytics
- full GitHub App integration

---

## 12. Success Criteria

The MVP is successful when:

```text
1. User opens a repo.
2. User creates a task.
3. App creates a worktree.
4. User runs Claude Code or Codex CLI inside the worktree.
5. App captures output.
6. App shows changed files and diff.
7. User runs tests/lint/build.
8. App shows verification result.
9. User reviews evidence.
10. App generates a useful PR report.
```

---

## 13. Future Roadmap

## V1

- GitHub CLI PR creation
- issue import
- richer risk reports
- more engine settings
- reusable task templates
- better artifact viewer

## V2

- MCP integration
- team sync
- optional cloud dashboard
- browser verification
- screenshots
- multi-agent workflows

## V3

- enterprise policy
- hosted runners
- more providers
- full agent marketplace
- deployment gates

---

## 14. Final Product Requirement

The product must make local AI coding agent work more structured, inspectable, and reviewable than raw terminal usage.
