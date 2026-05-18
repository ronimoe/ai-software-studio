# Architecture Diagrams

This folder contains Mermaid diagrams for **AI Software Studio**.

Diagrams are stored as Markdown files so they can be reviewed in Git, rendered in GitHub, and kept close to the architecture documentation.

## Diagram Index

| File | Purpose |
|---|---|
| `01-system-context.md` | Shows the high-level relationship between the developer, AI Software Studio, local engines, Git repo, SQLite, and artifacts |
| `02-container-architecture.md` | Shows the internal system layers: Next.js UI, Tauri bridge, Rust native core, services, local engines, Git, and storage |
| `03-runtime-flow.md` | Shows the end-to-end task execution workflow |
| `04-engine-adapter-flow.md` | Shows how Claude Code and Codex CLI are connected through a common engine adapter layer |
| `05-data-storage.md` | Shows how SQLite, local artifacts, logs, reports, and Git worktrees are organized |
| `06-task-lifecycle.md` | Shows the lifecycle of a task from draft to review, approval, rejection, or PR preparation |

## Diagram Rules

- Keep each diagram focused on one idea.
- Do not overload one diagram with every system detail.
- Prefer clear names over technical abbreviations.
- Update diagrams when architecture decisions change.
