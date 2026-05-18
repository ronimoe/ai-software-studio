# Changelog

All notable changes to AI Software Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.2] — 2026-05-18

Projects now persist. Opening a git repository writes it to a local SQLite database so it survives app restarts, and the dashboard remembers which project you last picked. This is the foundation for Plan 2 (task model + intake wizard).

### Added

- **SQLite persistence** via `sqlx` — `~/Library/Application Support/AI Software Studio/app.db` on macOS, equivalent platform-data dir on Linux. Schema initialized at startup; in-memory pool helper for tests.
- **`ProjectRepository`** — typed CRUD over the `projects` table with parameterized queries. Full TDD coverage (5 tests: insert/list/get + duplicate-path + not-found).
- **`open_project` Tauri command** — validates the selected path is a git working tree via `git rev-parse --show-toplevel`, canonicalizes to the repo root (so picking a subdirectory still opens the right project), and is idempotent on the same path. 4 tests covering happy path, non-git rejection, missing path, and idempotency, plus a regression test for subdirectory canonicalization.
- **`tauri-plugin-dialog`** for the native directory picker, exposed to the frontend via `dialog:default` capability.
- **Project switcher UI** — header dropdown lists all persisted projects with their paths and adds an "Open project…" action that fires the native picker. In `pnpm dev` mock mode, falls back to a `window.prompt()` so the dev experience stays usable without Tauri.
- **`useOpenProject` mutation hook** — invalidates the projects query and sets the new project as active on success.
- **`useMounted` hook** rewritten with `useSyncExternalStore` to satisfy the new `react-hooks/set-state-in-effect` rule from `eslint-plugin-react-hooks` v7.

### Changed

- **`ProjectService` now backed by SQLite** instead of the in-memory `fixtures::projects()` mock. `ProjectService::new` takes a `Db`.
- **`AppState::init` is async** — initializes the SQLite pool synchronously inside Tauri's `setup()` via `block_on`, so the first IPC from the webview can't race past the managed-state registration.
- **`activeProjectId` is `string | null`** — defaults to `null` until a project is picked or auto-selected from the persisted list. The dashboard auto-selects the first project on load if any exist.
- **`run()` in `lib.rs`** gated `#[cfg(not(test))]` so `tauri::generate_context!()` (which needs `out/` to exist) doesn't fire during `cargo test --lib` or `pnpm gen:bindings`.

### Fixed

- **`pnpm gen:bindings` regression** from PR #5 (specta-typescript 0.0.7 → 0.0.12) reverted via PR #10. The 0.0.12 bump silently broke the binding-export test because it forbids exporting BigInt-style integers, and `AppError.details: serde_json::Value` transitively contains `serde_json::Number` with `i64`.
- **`pnpm lint` regression** from PR #7 (eslint-config-next 15 → 16) fixed via PR #11. Migrated `eslint.config.mjs` off the `FlatCompat` shim to eslint-config-next 16's native flat config exports.

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

### Known limitations

- No real engine adapters yet — `claude-code` and `codex-cli` integration is stubbed via mock data.
- No SQLite persistence yet; services hold data in memory.
- No git worktree management, process runner, or verification execution wired through to the UI.
- macOS + Linux only; Windows is intentionally out of scope for the initial milestone.

[Unreleased]: https://github.com/ronimoe/ai-software-studio/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.2
[0.0.1]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.1
