# Changelog

All notable changes to AI Software Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5] — 2026-05-25

Click **Start** on a `WorktreeCreated` task and the app now spawns the real `claude` binary inside the worktree, streams its stdout/stderr line-by-line to a live terminal view, and lets you hit **Stop** to terminate it (SIGTERM, then SIGKILL after 2s). The agent reads the managed `CLAUDE.md` from Plan 3 as its priming context. This is the first plan that makes the app actually do something — every prior plan was scaffolding for this moment.

### Added

- **`detect_claude`** (`src-tauri/src/engines/detection.rs`) — real PATH scan + `claude --version` shell + semver-ish parse. Returns `EngineStatus { status: Ready | Detected | NotInstalled, version, binary_path }`. `Ready` means binary found AND version parsed; `Detected` means binary found but version unparseable. `EngineService::detect` calls it via `tokio::task::spawn_blocking` so the sync `which`/`Command::output` doesn't stall the runtime. 3 tests, gated by a `static Mutex<()>` because the tests mutate process-global PATH and cargo runs them in parallel by default.
- **`ProcessRunner`** (`src-tauri/src/process/mod.rs`) — owns a `dashmap::DashMap<task_id, Arc<Mutex<tokio::process::Child>>>` registry, spawns processes with piped stdio + `kill_on_drop`, and dispatches three tokio tasks per spawn: one forwarding stdout lines, one forwarding stderr lines, and a reaper that `wait()`s on the child, emits `task-exit`, and unregisters. `stop` sends SIGTERM via `libc::kill`, polls the registry for 2s, then falls back to `child.kill().await` (SIGKILL). `stop` on an unknown task_id is a no-op. 3 tests cover spawn+auto-cleanup, stop kills long-running, and unknown-task no-op.
- **`ClaudeCodeAdapter`** (`src-tauri/src/engines/adapters/claude_code.rs`) — thin wrapper that builds the argv (`--print <prompt>`) and delegates to `ProcessRunner::spawn`. Prompt is intentionally minimal — it tells the agent to read `CLAUDE.md`, follow the rules and the linked `.aistudio/task-brief.md`, write a failing test first, stay inside the worktree, and summarize on done. All of the per-task content lives in the managed `CLAUDE.md` that Plan 3 writes.
- **`TaskOutput` and `TaskExit` event types** (`src-tauri/src/process/mod.rs`) — registered via `tauri-specta`'s `collect_events![]` so they appear in `lib/bindings.ts` with typed `events.taskOutput.listen(cb)` and `events.taskExit.listen(cb)` helpers. Struct names are kebab-cased into the event identifiers (`task-output`, `task-exit`).
- **`start_task` / `stop_task` / `get_run_status` Tauri commands** (`src-tauri/src/commands/runs.rs`) — `start_task` validates the task is in `WorktreeCreated`/`Stopped`/`Failed`, looks up the worktree path, detects the `claude` binary on PATH, delegates to `ClaudeCodeAdapter::start`, then transitions the task to `Running`. `stop_task` calls `ProcessRunner::stop` and transitions to `Stopped`. `get_run_status` returns `{ taskId, running }`.
- **`AppState` holds `Arc<ProcessRunner>`** (`src-tauri/src/state.rs`) — and `lib.rs` setup wires `process.set_handle(app_handle)` so the runner can emit Tauri events. Setup is now async-spawned (was `block_on`) because `set_handle` is async.
- **`useDetectEngines` hook** (`features/engines/use-detect-engines.ts`) — TanStack Query against `tauri.detectEngines()` with 60s staleTime.
- **`useStartTask` + `useStopTask` mutations** (`features/runs/use-start-task.ts`, `use-stop-task.ts`) — invalidate `["tasks", projectId]` and `["task", id]` on success so the panel re-fetches the new status.
- **`useTaskOutput` hook** (`features/runs/use-task-output.ts`) — subscribes to `events.taskOutput.listen()` and `events.taskExit.listen()`, filters by `taskId`, exposes `{ lines: TerminalLine[], exitCode: number | null | undefined }`. Wrapped in try/catch so dev mode (no Tauri runtime) doesn't crash; events just don't fire.
- **`TerminalView` component** (`components/panels/agent-workspace/terminal-view.tsx`) — renders streamed lines (stderr in amber), auto-scrolls on append, shows a `— process exited (code N) —` footer when `exitCode` is defined. Mounted with `key={task.id}` so React remounts on task switch (replaces an in-effect state reset that the new `react-hooks/set-state-in-effect` lint rule flags).
- **`startTask` / `stopTask` / `getRunStatus` mocks** (`lib/tauri.ts`) — so `pnpm dev` continues to exercise the new UI without compiling Rust.

### Changed

- **`StartButton` renders status-aware variants** (`components/panels/agent-workspace/start-button.tsx`) — `Create worktree` (icon: GitBranch) when status is `Draft`, `Start` (icon: Play) when `WorktreeCreated`/`Stopped`/`Failed`, `Stop` (icon: Square, destructive variant) when `Running`/`VerificationRunning`. Single component, three behaviors.
- **`EngineService::detect` returns real data** (`src-tauri/src/engines/mod.rs`) — was returning the `fixtures::engines()` mock. Now calls `detect_claude` via `spawn_blocking`.
- **`AgentWorkspacePanel` shows live `TerminalView`** (`components/panels/agent-workspace/index.tsx`) — when the task status is anything other than `Draft`, a "Live Output" section renders the terminal between Acceptance Criteria and Activity Log.

### Dependencies

- `dashmap = "6"` — lockless task_id→Child registry inside `ProcessRunner`.
- `libc = "0.2"` — `libc::kill(pid, SIGTERM)` for graceful process termination.
- `tokio` dev-deps gain `"time"` — tests need `sleep`/`Duration`.

### Notes

- The event payload struct names `TaskOutput` and `TaskExit` were chosen so `tauri-specta`'s `#[derive(Event)]` macro (which kebab-cases the struct name into the event identifier) produces `"task-output"` and `"task-exit"` — matching the strings the Rust code actually emits.
- Detection tests serialize PATH mutations through a `static Mutex<()>`. Cargo's default parallel test runner races on the global PATH otherwise (confirmed: 1-in-10 flake without the lock).

## [0.0.4] — 2026-05-19

The "Create worktree" button is live. Clicking it on a draft task spawns a real git worktree under `~/Library/Application Support/AI Software Studio/worktrees/{project}/{task}` off the project's default branch, drops a managed `CLAUDE.md` and `.aistudio/task-brief.md` into it, and transitions the task to `WorktreeCreated`. If anything goes wrong mid-creation, the worktree, its branch ref, and any installed files are rolled back so the user's repo is left exactly as it was. This is the project's first compensating-action operation — the template Plans 4–7 will follow.

### Added

- **`GitService`** (`src-tauri/src/git/mod.rs`) — wraps the `git` CLI via `std::process::Command` (no `git2` dependency for the v0.1 surface). `worktree_add(repo, branch, dest, base_ref)` creates a fresh branch from the specified ref and adds a worktree at the dest. `worktree_remove(repo, dest)` uses `git worktree remove --force` plus a `fs::remove_dir_all` fallback so double-cleanup is safe. `branch_delete(repo, branch)` swallows "not found" so it's idempotent like remove. 4 git-service tests, including an explicit idempotency assertion that the rollback path depends on.
- **`worktree_paths` helpers** (`src-tauri/src/git/worktree_paths.rs`) — `worktree_root()` returns `~/Library/Application Support/AI Software Studio/worktrees/`. `worktree_path(project_id, task_id)` is the canonical layout. `branch_name(task_id)` produces `aistudio/task-{first-8-chars}` so branch names stay readable. `is_within_worktree_root(path)` canonicalizes both sides when the path exists (defeating symlinks and `..` tricks) and falls back to a string-prefix check otherwise — used to guard `remove_worktree` against a corrupted SQLite row pointing outside the managed root. 9 path tests.
- **`WorktreeContextService`** (`src-tauri/src/core/worktree_context.rs`) — `install(worktree, task, brief)` writes `.aistudio/task-brief.md`, writes/updates `CLAUDE.md` with a managed section bracketed by `<!-- aistudio:begin -->` / `<!-- aistudio:end -->` markers, an `@.aistudio/task-brief.md` import, and 4 task rules, and ensures `.aistudio/` is in the worktree's `.gitignore`. All operations are idempotent: existing CLAUDE.md content is preserved, a second install replaces the managed section instead of duplicating, and the gitignore line is added only once. 7 context tests including a deliberate failure-mode assertion the orchestrator's rollback contract relies on.
- **`core::worktree_lifecycle::create_worktree_lifecycle`** (`src-tauri/src/core/worktree_lifecycle.rs`) — the orchestrator. Takes `base_ref` and threads it to `worktree_add`. Performs LIFO compensating-action cleanup on partial failure across all four steps: any post-add failure calls `rollback_worktree(repo, dest, branch)` which `worktree_remove`s the worktree, then `branch_delete`s the branch ref, and (for step-4) reverts task status to Draft. Step-1 failure also rolls back because `git worktree add -b` creates the branch ref before validating the destination. Pure of `State<'_, AppState>` so it can be unit-tested with real services against a temp repo + in-memory DB. 1 lifecycle test verifies no DB residue on step-1 failure.
- **`create_worktree` + `remove_worktree` Tauri commands** (`src-tauri/src/commands/worktrees.rs`) — thin wrappers around the lifecycle. `create_worktree` rejects non-Draft tasks and existing dest paths, then delegates. `remove_worktree` validates the worktree path is under the managed root via `is_within_worktree_root` before any destructive operation, then `worktree_remove`s + `branch_delete`s + clears the task fields + reverts status to Draft.
- **`TaskService::set_branch_and_worktree` and `clear_worktree`** (`src-tauri/src/tasks/{mod.rs,repository.rs}`) — sqlx UPDATE wrappers used by the lifecycle to persist or clear the worktree fields on the task row.
- **`AppState` holds `GitService` and `WorktreeContextService`** (`src-tauri/src/state.rs`) — both stateless unit structs in v0.1, held on `AppState` for API symmetry as future statefulness lands.
- **`useCreateWorktree` mutation hook** (`features/worktrees/use-create-worktree.ts`) — unwraps `Result<Task, AppError>` and invalidates `["tasks", projectId]` + `["task", taskId]` query keys on success so the panel refreshes automatically.
- **`StartButton` component** (`components/panels/agent-workspace/start-button.tsx`) — the "Create worktree" button with a `Loader2` spinner during the mutation and a `GitBranch` icon at rest. Rendered only on tasks with status `Draft`; the existing Approve/Request Changes pair takes over once the worktree exists.
- **Worktree path display** in Agent Workspace — when a task has a `worktreePath`, it renders under the description in monospace so the developer can see where the agent will work.
- **`createWorktree` + `removeWorktree` mocks** (`lib/tauri.ts`) — so `pnpm dev` continues to exercise the UI without compiling Rust. The mock branch name uses the same `aistudio/task-{first-8-chars}` formula as the real backend.
- **Failure-semantics architectural rule** (`docs/superpowers/plans/2026-05-18-v0.1-plan-3-worktree-and-context.md` §Architecture) — codifies when to use compensating-action cleanup (stateful externally-tracked side effects like git worktrees, OS processes, GitHub PRs) vs. accept partial state (regenerable side effects derived from DB state, like Plan 2's task-brief file). This is the template Plans 4–7 should follow for new stateful operations.

### Changed

- **`commands/worktrees::create_worktree` is a thin wrapper** around `core::worktree_lifecycle::create_worktree_lifecycle`. The heavy lifting lives in an orchestrator decoupled from `State<'_, AppState>` so it can be unit-tested against a real temp repo + in-memory DB.
- **`worktree_add` takes an optional base ref**. Previously it ran `git worktree add -b <branch> <dest>` which uses current HEAD — if the developer happened to have a feature branch checked out in their project repo when they clicked "Create worktree", the agent's worktree branched off that, not off `main`. Now the lifecycle threads `&project.default_branch` through so worktrees always start from the configured default.

### Fixed

- **Rollback now deletes the branch ref**. The earlier compensating-action draft only removed the worktree directory; `git worktree add -b` creates a real branch ref in the parent repo that survives `worktree_remove`. After any post-add failure (and any step-1 failure, because git creates the ref before validating the destination), the next retry would fail with "fatal: a branch named ... already exists" and the user was wedged. `branch_delete` is now part of the LIFO cleanup.
- **`remove_worktree` can no longer be used as arbitrary directory deletion**. The command guards with `is_within_worktree_root(&dest)` before calling `worktree_remove`; a corrupted or imported SQLite row with `worktree_path = "/Users/me"` is rejected with `invalid_arg` instead of being passed to `fs::remove_dir_all`.
- **Version drift** — `VERSION` file was at `0.0.2` while `package.json` + `Cargo.toml` + `tauri.conf.json` were at `0.0.3` (Plan 2's bump missed `VERSION`). All four files now sit at `0.0.4`.

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

[Unreleased]: https://github.com/ronimoe/ai-software-studio/compare/v0.0.4...HEAD
[0.0.4]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.4
[0.0.3]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.3
[0.0.2]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.2
[0.0.1]: https://github.com/ronimoe/ai-software-studio/releases/tag/v0.0.1
