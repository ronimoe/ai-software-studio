# AI Software Studio Documentation

This folder contains the main product, architecture, decision, diagram, and exploration documentation for **AI Software Studio**.

AI Software Studio is a local-first desktop application that helps developers orchestrate local AI coding agents such as **Claude Code** and **Codex CLI** through a structured, evidence-backed software development workflow.

The product is designed to help a developer move from unstructured AI coding sessions into a controlled workflow:

```text
Task
→ Constraints
→ Isolated Git worktree
→ Local agent execution
→ Terminal/activity capture
→ Changed files
→ Verification
→ Review
→ PR evidence report
```

---

## Product Summary

**AI Software Studio** is a local command center for AI coding agents.

It does not directly call OpenAI or Anthropic APIs in the initial architecture. Instead, it uses the developer’s locally installed tools such as Claude Code and Codex CLI. Those tools handle their own authentication, subscription, and usage limits.

The app provides the workflow layer around those tools:

- task definition
- acceptance criteria
- constraints
- local engine selection
- Git worktree isolation
- terminal output capture
- changed file tracking
- verification checks
- review room
- risk detection
- PR evidence report generation

---

## Documentation Structure

```text
docs/
├── README.md
├── product-brief.md
├── product-spec.md
├── architecture/
│   ├── README.md
│   ├── architecture.md
│   ├── adr/
│   │   ├── README.md
│   │   ├── 0001-use-local-first-architecture.md
│   │   ├── 0002-use-tauri-rust-nextjs-stack.md
│   │   ├── 0003-support-macos-and-linux-only-initially.md
│   │   ├── 0004-use-local-cli-engine-adapters.md
│   │   └── 0005-use-git-worktrees-for-task-isolation.md
│   └── diagrams/
│       ├── README.md
│       ├── 01-system-context.md
│       ├── 02-container-architecture.md
│       ├── 03-runtime-flow.md
│       ├── 04-engine-adapter-flow.md
│       ├── 05-data-storage.md
│       └── 06-task-lifecycle.md
└── exploration/
    ├── README.md
    ├── 01-terminal-pty-options.md
    ├── 02-engine-adapter-spike.md
    ├── 03-tauri-nextjs-packaging-spike.md
    ├── 04-github-integration-options.md
    ├── 05-artifact-storage-options.md
    └── 06-mcp-integration-options.md
```

---

## Core Product Documents

| File | Purpose |
|---|---|
| `manual.md` | End-user manual for the **current shipped version**. What works today, where data lives, troubleshooting. Honest about what's still mocked. Update on every plan release. |
| `product-brief.md` | Explains the product vision, positioning, target users, problem, value proposition, differentiation, and strategy |
| `product-spec.md` | Defines product behavior, user flows, modules, MVP scope, requirements, and implementation-facing product details |

Start with these documents if you want to understand **what the product is** and **why it should exist**. Read `manual.md` if you want to actually use the build that's on your machine.

---

## Architecture Documents

| Path | Purpose |
|---|---|
| `architecture/architecture.md` | Main system architecture document |
| `architecture/README.md` | Guide to the architecture folder |
| `architecture/adr/` | Architecture Decision Records |
| `architecture/diagrams/` | Mermaid architecture diagrams |

Use this folder to understand **how the system is designed**.

---

## Recommended Reading Order

For a new contributor or future review, read in this order:

1. `product-brief.md`
2. `product-spec.md`
3. `architecture/architecture.md`
4. `architecture/diagrams/README.md`
5. `architecture/adr/README.md`
6. `exploration/README.md`

This order explains:

```text
Why the product exists
→ What it should do
→ How it is built
→ Why major decisions were made
→ What is still being explored
```

---

## Current Product Direction

### Initial Product Type

```text
Local-first desktop application
```

### Initial Target Platforms

```text
macOS
Linux
```

Windows is intentionally excluded from the initial scope to reduce complexity around:

- PowerShell
- Windows path handling
- ConPTY
- WSL ambiguity
- Windows-specific process management
- CLI behavior differences

---

## Current Technology Direction

| Layer | Technology |
|---|---|
| Frontend UI | Next.js + React + TypeScript |
| Desktop shell | Tauri |
| Native runtime | Rust |
| Local database | SQLite |
| Rust database access | sqlx |
| Styling | Tailwind CSS + shadcn/ui |
| State management | Zustand / TanStack Query |
| Diff viewer | Monaco Editor or react-diff-view |
| Local execution | Rust process runner + Unix PTY |
| Git integration | Git CLI first |
| Agent engines | Claude Code CLI, Codex CLI |

---

## Core Design Principles

### 1. Local-first

The app should run locally by default.

The initial product should not depend on a hosted backend for core functionality.

### 2. No provider token storage

The app should not store OpenAI or Anthropic API tokens in the initial architecture.

Claude Code and Codex CLI handle their own authentication.

### 3. Human approval first

AI agents can execute, but humans remain responsible for:

- intent
- constraints
- review
- approval
- final merge/deployment decision

### 4. Evidence-backed workflow

The app should not rely only on agent claims.

The app should collect independent evidence:

- changed files
- Git diff
- terminal logs
- verification results
- risk reports
- PR evidence reports

### 5. Isolated task execution

Each task should run inside its own Git worktree.

Agents should not directly modify the user’s main working tree.

### 6. Engine-agnostic design

Claude Code and Codex CLI are initial engines, but the architecture should allow future engines.

---

## MVP Definition

The MVP is successful when a user can:

```text
1. Open a local Git repository.
2. Detect Claude Code or Codex CLI.
3. Create a structured task.
4. Create an isolated Git worktree.
5. Run the selected engine inside the worktree.
6. See live terminal output.
7. See changed files.
8. Inspect diff.
9. Run verification commands.
10. See pass/fail results.
11. Generate a PR evidence report.
```

---

## Core Product Sentence

> AI Software Studio is a local-first command center that lets developers safely delegate coding tasks to Claude Code and Codex CLI, then review verified evidence before approving the work.
