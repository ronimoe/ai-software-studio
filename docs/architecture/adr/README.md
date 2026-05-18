# Architecture Decision Records

This folder contains Architecture Decision Records, also called ADRs, for **AI Software Studio**.

ADRs are used to document important architecture decisions that have been accepted or are being seriously proposed.

## Purpose

Use ADRs to explain:

- what decision was made
- why the decision was made
- what alternatives were considered
- what tradeoffs were accepted
- when the decision should be revisited

ADRs should be short, practical, and easy to read.

## Current ADRs

| ADR | Title | Status |
|---|---|---|
| `0001-use-local-first-architecture.md` | Use Local-First Architecture | Accepted |
| `0002-use-tauri-rust-nextjs-stack.md` | Use Tauri + Rust + Next.js Stack | Accepted |
| `0003-support-macos-and-linux-only-initially.md` | Support macOS and Linux Only Initially | Accepted |
| `0004-use-local-cli-engine-adapters.md` | Use Local CLI Engine Adapters | Accepted |
| `0005-use-git-worktrees-for-task-isolation.md` | Use Git Worktrees for Task Isolation | Accepted |

## ADR Status Values

| Status | Meaning |
|---|---|
| `Proposed` | Under consideration |
| `Accepted` | Current active decision |
| `Rejected` | Considered but not chosen |
| `Superseded` | Replaced by a newer ADR |
| `Deprecated` | No longer recommended, but not directly replaced |

## ADR Workflow

```text
Explore
→ Prototype
→ Decide
→ Create ADR
→ Update architecture.md
```
