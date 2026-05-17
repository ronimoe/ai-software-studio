# Changelog

All notable changes to AI Software Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] — 2026-05-18

Initial scaffold. Establishes the architectural foundation; no agent execution yet.

### Added

- **Tauri + Next.js + Rust scaffold** — desktop shell with static-export Next.js frontend served from the Tauri window.
- **Typed Tauri command bridge** — `specta` / `tauri-specta` generate `lib/bindings.ts` from Rust types so the TS frontend and Rust core share one source of truth. Wired via `pnpm gen:bindings`.
- **Dual runtime in `lib/tauri.ts`** — single `tauri` client that delegates to real Tauri commands inside the desktop window and to in-process mocks (`lib/mock-data.ts`) during plain browser dev. `pnpm dev` works without compiling Rust.
- **Rust core skeleton** — module layout for `commands/`, `engines/`, `git/`, `process/`, `verification/`, `policy/`, `artifacts/`, `db/`, `projects/`, `tasks/`, plus `state::AppState` aggregating services.
- **Domain model** — `Project`, `Task` (with `TaskStatus` lifecycle, acceptance criteria, constraints, `RiskLevel`), `EngineStatus`, `VerificationRun` defined in `src-tauri/src/models.rs` and exported to TS.
- **Dashboard UI shell** — `DashboardShell` 3-column grid (280px / 1fr / 360px) wired with seven panels: Task Board, Engineering Snapshot, Agent Workspace, Review Room, Context Graph, Conversation, Agent Manager.
- **State + data layer** — Zustand stores for ephemeral UI state, TanStack Query for server state, feature hooks under `features/*/use-*.ts`.
- **shadcn/ui primitives** (`components/ui/`) on Tailwind v4 with a dark-default theme via `next-themes`.
- **Vitest + jsdom** test setup with a smoke test for the Tauri bridge.
- **Product, architecture, and exploration docs** in `docs/` (product brief, product spec, architecture doc, five ADRs, six Mermaid diagrams, exploration spikes).

### Known limitations

- No real engine adapters yet — `claude-code` and `codex-cli` integration is stubbed via mock data.
- No SQLite persistence yet; services hold data in memory.
- No git worktree management, process runner, or verification execution wired through to the UI.
- macOS + Linux only; Windows is intentionally out of scope for the initial milestone.

[Unreleased]: https://github.com/ronimoe/ai-software-studio/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.1
