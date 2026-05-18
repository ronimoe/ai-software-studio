# Changelog

All notable changes to AI Software Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3] — 2026-05-18

Tasks now exist as real, persisted things. A 6-step intake wizard lives behind the **+** button on the Task Board: title → description → acceptance criteria → constraints → out-of-scope → files-to-touch → review. Creating a task writes a row to SQLite, renders a `task-brief.md` artifact to disk, and selects the new task in the workspace.

### Added

- **sqlx migrations infrastructure** (`src-tauri/migrations/`) — versioned `.sql` files replace the single `schema.sql`. `Db::init` and `Db::test_pool` both call `sqlx::migrate!("./migrations").run(&pool)`, so test DBs see the same schema as production. Plan 1's projects table moves into `20260101000000_initial_schema.sql`.
- **Task model schema** in `20260102000000_task_model.sql` — `tasks`, `task_acceptance_criteria`, `task_constraints` tables with `ON DELETE CASCADE` from tasks; positional ordering on the two child tables. Sub-second `tasks.created_at` (`strftime('%Y-%m-%dT%H:%M:%fZ', 'now')`) plus `rowid DESC` tiebreaker for deterministic ordering across rapid inserts.
- **`TaskRepository`** (`src-tauri/src/tasks/repository.rs`) — transactional `insert` (writes task row + ordered criteria + ordered constraints in one tx), `list_for_project`, `get` (hydrates relations), `update_status`. 6 TDD tests against in-memory SQLite covering insert, ordering, project isolation, hydration, not-found, status persistence.
- **`TaskBriefService`** (`src-tauri/src/tasks/brief.rs`) — pure `render_brief(&Task) -> String` produces a Markdown brief with H1 title, conditional Description/Acceptance Criteria/Constraints/Out of Scope/Files to Touch sections, and a footer. `write_brief(&Task) -> Result<(), AppError>` drops the rendered string into the durable artifact store. 9 TDD snapshot-style tests covering each section.
- **`artifact_dir(project_id, task_id)`** (`src-tauri/src/artifacts/mod.rs`) — resolves to `~/Library/Application Support/AI Software Studio/projects/{project_id}/tasks/{task_id}/artifacts/` and `create_dir_all`s eagerly.
- **`create_task` Tauri command** wired through tauri-specta. Registered in both `collect_commands![...]` lists in `lib.rs`. The mock implementation in `lib/tauri.ts` mirrors the real command shape for browser-only dev.
- **shadcn primitives** — `dialog`, `textarea`, `input`, `label` installed under `components/ui/`. Use the consolidated `radix-ui` meta-package (already in deps), so no new `@radix-ui/react-*` packages were added.
- **Wizard state reducer** (`components/panels/task-board/intake-state.ts`) — `IntakeForm` interface, `STEPS` const tuple driving a literal-typed `StepId`, `isStepValid(step, form)` step gating (title required, ≥1 acceptance criterion required, optional steps pass through), `splitLines` helper that trims and drops empty lines (handles CRLF too via `.trim()`). 19 tests.
- **`useCreateTask` mutation hook** (`features/tasks/use-create-task.ts`) — wraps `tauri.createTask`, throws on `AppError`, invalidates `["tasks", projectId]` on success so the task list refetches automatically.
- **`NewTaskDialog` wizard UI** (`components/panels/task-board/new-task-dialog.tsx`) — 6-step modal with type-narrowed setters, deferred form reset on close, autoFocus on each step's first input, Loader2 spinner during submit, type-safe review screen rendering each field with bullet-list line splitting for acceptance/constraints.
- **+ button on Task Board** wired to open the wizard, disabled when no project is active. Empty/loading/no-project states render appropriate copy.
- **`docs/` checked into the repo** — architecture notes, ADRs, product brief/spec, exploration spikes, and the v0.1 plan files now ship with the source so worktrees and future contributors share the same context.

### Changed

- **`Task` model gains `out_of_scope` and `files_to_touch_hint`** — both `String` fields between `description` and `acceptance_criteria`. Mirrored in `CreateTaskRequest` (new struct in `models.rs`), the SQLite schema, the TypeScript binding, and the 5 fixture tasks. Field order in Rust drives field order in generated TS via tauri-specta.
- **`TaskService` is now a thin pass-through** to `TaskRepository` plus `TaskBriefService::write_brief` on `create`. Insert-first ordering means a brief-write failure leaves the task persisted with no on-disk brief; the DB is the source of truth and `render_brief` can regenerate from a `Task`.
- **`AppState::init` passes `db.clone()` to `TaskService::new`** so the service can talk to SQLite.
- **`tsconfig.json` auto-updated by Next.js 16** — `jsx: "react-jsx"` (mandatory for Next 16's Turbopack JSX compile) and a new `.next/dev/types/**/*.ts` includes entry.

### Fixed

- **Infinite-loop crash in `ActivityLog`** (`components/panels/agent-workspace/activity-log.tsx`) — `(s) => s.streamingLog[taskId] ?? []` returned a fresh empty array every render, tripping React's `useSyncExternalStore` snapshot check and throwing "Maximum update depth exceeded" the moment a task became active. Hoisted `EMPTY_LINES` to a module-level constant for a stable reference. Latent in Plan 1 (no way to have an active task); surfaced by Plan 2's wizard activating tasks on create.

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

[Unreleased]: https://github.com/ronimoe/ai-software-studio/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.3
[0.0.2]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.2
[0.0.1]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.1
