# Exploration Notes

This folder contains exploration notes for **AI Software Studio**.

Exploration notes are used for topics that are still uncertain, experimental, or not ready to become Architecture Decision Records.

## Difference Between Exploration Notes and ADRs

| Type | Purpose | Status |
|---|---|---|
| Exploration Note | Investigate uncertain topics | Flexible / temporary |
| ADR | Record accepted architecture decisions | Formal / durable |

## Current Exploration Files

| File | Purpose |
|---|---|
| `01-terminal-pty-options.md` | Explores how to run and control interactive terminal agents |
| `02-engine-adapter-spike.md` | Explores how to abstract Claude Code, Codex CLI, and future engines |
| `03-tauri-nextjs-packaging-spike.md` | Explores how to package Next.js inside Tauri |
| `04-github-integration-options.md` | Explores manual PR reports, GitHub CLI, OAuth, and GitHub App options |
| `05-artifact-storage-options.md` | Explores how to store logs, diffs, reports, and metadata |
| `06-mcp-integration-options.md` | Explores future MCP integration possibilities |

## Workflow

```text
Question
→ Explore options
→ Build small prototype
→ Record findings
→ Recommend direction
→ Create ADR if needed
```
