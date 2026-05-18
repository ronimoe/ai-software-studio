# Architecture Documentation

This folder contains the main architecture documentation for **AI Software Studio**.

AI Software Studio is designed as a local-first desktop application for orchestrating local AI coding agents such as Claude Code and Codex CLI.

## Contents

```text
architecture/
├── README.md
├── architecture.md
├── adr/
└── diagrams/
```

## Files and Folders

| Path | Purpose |
|---|---|
| `architecture.md` | Main architecture document describing system goals, stack, modules, services, data storage, and runtime flow |
| `adr/` | Architecture Decision Records for accepted technical decisions |
| `diagrams/` | Mermaid diagrams for system context, container architecture, runtime flow, engine adapters, storage, and task lifecycle |

## How to Use This Folder

Start with:

```text
architecture.md
```

Then review:

```text
diagrams/
```

for visual explanations.

Use:

```text
adr/
```

to understand why major architecture decisions were made.

## Current Architecture Summary

```text
Next.js UI
  ↓
Tauri command bridge
  ↓
Rust native core
  ↓
Local engines, Git worktrees, SQLite, artifacts
```

## Current Architecture Decisions

The system currently assumes:

- Local-first architecture
- macOS and Linux support only
- Tauri + Rust + Next.js stack
- SQLite for local persistence
- Claude Code and Codex CLI as local engines
- Git worktrees for task isolation
- No direct provider API token storage
