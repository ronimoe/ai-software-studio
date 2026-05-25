# Changelog

All notable changes to AI Software Studio will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-05-25

**v0.1 milestone.** The loop closes end-to-end: pick a task, get an isolated git worktree, run the engine in a live terminal, see exactly which files changed and their diffs, run install/typecheck/lint/test/build verification with persisted results, copy a Markdown evidence report to the clipboard, and open a real GitHub PR via `gh` — all from one window. This release bundles Plans 5, 6, and 7 plus a hotfix for a Dependabot-induced specta drift.

### Added

#### Changed Files panel + diff viewer (Plan 5)

- **`git::status`** (`src-tauri/src/git/status.rs`) — parses `git status --porcelain=v1 -z` into `Vec<ChangedFile>` with NUL-separated record handling and a `classify(X, Y)` table that maps status pairs to `Added`/`Modified`/`Deleted`/`Renamed`/`Untracked`/`Conflicted`. Renamed records consume two NUL-separated entries (original + new). 7 tests covering clean repo, modified/added/deleted/untracked, multi-file ordering, and the "not a git dir" error path.
- **`git::diff`** (`src-tauri/src/git/diff.rs`) — `git diff --no-color HEAD -- <path>` for tracked files. For untracked files, runs `git add --intent-to-add <path>` first so the file shows up in the diff, then `git reset -- <path>` to unstage the intent-to-add marker — leaving the worktree state unchanged. 3 tests for modified, untracked-via-intent-to-add, and the empty-for-unchanged invariant.
- **`get_changed_files` + `get_file_diff` Tauri commands** (`src-tauri/src/commands/diffs.rs`) — both look up the task, error on `task has no worktree`, and `spawn_blocking` the git call. `ChangedFile` + `ChangeStatus` exported as cross-boundary types.
- **`reconcile_after_exit` command** (`src-tauri/src/commands/runs.rs`) — called from the frontend when the agent process exits. Reads `git status` against the worktree; if there are changes, transitions `Running → ReviewReady`; if clean, transitions to `Stopped`. No-op if the task is not currently Running. Worktree-less tasks transition to Stopped.
- **`useChangedFiles`, `useFileDiff`, `useReconcileAfterExit` hooks** (`features/diffs/*`, `features/runs/use-reconcile-after-exit.ts`) — TanStack Query bindings; `useChangedFiles` polls every 5 s. `TerminalView` fires `reconcileAfterExit.mutate(taskId)` in a `useEffect` when its `exitCode` flips from `undefined` to a value.
- **`ChangedFilesPanel`** (`components/panels/agent-workspace/changed-files-panel.tsx`) — lists each changed file with a lucide-react status icon (FilePlus/FileEdit/FileMinus/FileQuestion/GitMerge), the path in monospace, and `+adds −dels` counters. Clicking a file opens the dialog. 4 Vitest specs cover loading, empty, list-render, dialog-open.
- **`DiffViewerDialog`** (`components/panels/agent-workspace/diff-viewer-dialog.tsx`) — shadcn `Dialog` wrapping `react-diff-view`'s split-view `Diff` + `Hunk`. Parses unified-diff text once via `parseDiff` inside `useMemo`. Loading/empty states match the panel. 5 specs covering closed/open/loading/empty/parsed-hunk rendering.
- **`get_changed_files` / `get_file_diff` / `reconcile_after_exit` mocks** (`lib/tauri.ts`) — three modified files, a plausible unified-diff string, and a mock task transition to `reviewReady`.

#### Verification runner + Review Room (Plan 6)

- **SQLite migration `20260103000000_verification.sql`** — three tables: `verification_runs (id, task_id, started_at, finished_at)`, `verification_checks (id, run_id, kind, status, duration_ms, log_excerpt, position)`, `app_settings (scope, key, value)` with `(scope, key)` as the composite primary key.
- **`verification::runner::run_single`** (`src-tauri/src/verification/runner.rs`) — shells `sh -c <command>` via `tokio::process::Command` with piped stdout+stderr, reads both streams concurrently in a `tokio::select!` loop into a single buffer, and **tail-bounds the buffer to 4 KB** via `cap_buf` (drains the front when the buffer exceeds 4096 bytes). Empty commands return `Skipped` without spawning. Returns `CheckResult { kind, status, duration_ms, log_excerpt }`. 5 tests covering pass/fail/excerpt-content/4KB-bound/skipped.
- **`verification::repository::VerificationRepository`** (`src-tauri/src/verification/repository.rs`) — transactional `insert_run(task_id, checks)` writes the run row, the ordered checks (with a `position` column), and a `finished_at` timestamp inside one transaction. `list_for_task` returns runs newest-first by `started_at DESC`. `get(id)` rehydrates a single run with its checks. 2 tests against the in-memory pool.
- **`verification::settings::SettingsRepository`** (`src-tauri/src/verification/settings.rs`) — per-project verification commands stored under `scope = 'project:{project_id}'`, `key = 'verification.{install|typecheck|lint|test|build}'`. `get_for_project` falls back to `VerificationSettings::default()` (`pnpm install`/`pnpm typecheck`/`pnpm lint`/`pnpm test`/`pnpm build`) when no rows exist. `set_for_project` deletes and reinserts inside a transaction (overwrite semantics). 3 tests covering defaults, round-trip, and overwrite.
- **`VerificationService::run_for_task`** — orchestrates the install → typecheck → lint → test → build sequence, skipping any command not configured. Each step runs sequentially, captures a result, and the full set is persisted as one run. Returns the rehydrated `VerificationRun`.
- **Four Tauri commands** (`src-tauri/src/commands/verification.rs`) — `list_verification` (rewritten to hit the repository), `run_verification` (looks up the worktree, calls the service), `get_verification_settings`, `set_verification_settings`.
- **`useRunVerification`** (`features/verification/use-run-verification.ts`) — mutation that invalidates `["verification", taskId]` on success. **`useVerificationSettings` + `useSetVerificationSettings`** (`features/verification/use-verification-settings.ts`) — query + mutation pair for per-project commands.
- **`RunVerificationButton`** (`components/panels/review-room/run-verification-button.tsx`) — outline-variant button gated on worktree existence, spinner while pending. 4 Vitest specs cover disabled/enabled/pending/click.
- **Review Room rewires its actions slot** — the button slots into the panel header alongside the status pills already rendered from the latest verification run.
- **Three new mocks** in `lib/tauri.ts` for verification commands.

#### GitHub push + `gh pr create` + evidence report (Plan 7)

- **`engines::github`** (`src-tauri/src/engines/github.rs`) — `detect()` calls `which_in("gh", $PATH)`, shells `gh auth status`, parses the `Logged in to github.com as <login>` line via `parse_account`, and returns `GitHubStatus { auth: Authed|NotAuthed|NotInstalled, binary_path, account }`. `push_branch(repo, branch)` runs `git push -u origin <branch>` from the worktree. `create_pr(repo, title, body, base, draft)` shells `gh pr create --title --body --base [--draft]` and pulls the PR URL from the first stdout line containing `github.com`. The `which_in` and `parse_account` helpers are `pub(super)` for testability.
- **`commands::pr::report::render`** (`src-tauri/src/commands/pr/report.rs`) — pure Markdown renderer composing the evidence report from `Task`, `&[ChangedFile]`, and an optional latest `VerificationRun`. Sections: H1 title, **Task** (title + id + description), **Acceptance Criteria** (with `[x]`/`[ ]` mirroring `satisfied`), **Files Changed** (Markdown table), **Verification** (table with emoji badges ✅/❌/⏭/⚠️/⏳/—), **Constraints** (only when present), and a footer with the task id. 6 tests covering header, files table, verification badges, constraints, satisfied criteria, and the empty-changes message.
- **Three Tauri commands** (`src-tauri/src/commands/pr/mod.rs`) — `detect_github` (async-wrapped `spawn_blocking`), `render_pr_report` (looks up task + verification + changed files, renders), `create_pr` (pushes the branch, renders the body, creates the PR, transitions the task to `PrPrepared`).
- **Three hooks** (`features/pr/*`) — `useDetectGithub` (60 s `staleTime` query), `useRenderPrReport` (mutation), `useCreatePr` (mutation, invalidates the task on success).
- **`CopyReportButton`** (`components/panels/review-room/copy-report-button.tsx`) — ghost-variant button that renders the report via the Tauri command, writes the result to the system clipboard with `@tauri-apps/plugin-clipboard-manager`'s `writeText`, and flashes a green check icon for 1.5 s. 3 Vitest specs covering idle/pending/click+clipboard.
- **`CreatePrButton`** (`components/panels/review-room/create-pr-button.tsx`) — `gh` auth-gated, with a `title` tooltip that explains the disabled reason (no worktree / not installed / not authed). On click, calls `useCreatePr` with `{ baseBranch: null, draft: false }`. On success, the button transforms into an "Open PR" external link to the returned URL. 6 Vitest specs cover all four disabled cases, the spinner, and the click → success → link transformation.
- **Review Room actions slot now holds three buttons** — `CopyReportButton`, `RunVerificationButton`, `CreatePrButton` — in a single flex row.
- **`@tauri-apps/plugin-clipboard-manager` + `tauri-plugin-clipboard-manager`** wired in, with `clipboard-manager:default` added to `capabilities/default.json`.
- **Three new mocks** in `lib/tauri.ts` for `detectGithub`, `renderPrReport`, `createPr`.

### Changed

- **`fixtures::verification_for_task` removed** (`src-tauri/src/fixtures.rs`). The old fixture returned a hand-rolled `VerificationRun` for `task-042` and was the only consumer of `VerificationCheck`/`VerificationStatus`/`VerificationRun` from the fixtures module. The new repository-backed `list_for_task` replaces it entirely.
- **`VerificationService::new` now takes `Db`** (`src-tauri/src/verification/mod.rs`, `src-tauri/src/state.rs`) so the service can own its repository and settings instances. `AppState::init` threads `db.clone()` to the constructor.
- **`Review Room` reads from `useTask`** to know if the active task has a worktree, and gates the verification + PR buttons accordingly.

### Fixed

- **`tokio::select!` double-mutable-borrow bug in `run_single`** — the plan as written shared a single `tmp: [u8; 1024]` buffer between the stdout and stderr `select!` branches, which compiled but tripped E0499 against `&mut tmp` in the second branch on stable. Split into `out_tmp` + `err_tmp`.
- **Specta drift after Dependabot PR #24** (post-merge fix landed via [PR #26](https://github.com/ronimoe/ai-software-studio/pull/26)) — `specta-typescript` got bumped from `0.0.7` → `0.0.11`, which cascaded `specta` to `2.0.0-rc.24`. rc.24 uses unstable Rust features (`debug_closure_helpers`, `const TypeId::of`) stabilized in 1.91, but `rust-toolchain.toml` pins us to 1.88. `src-tauri/Cargo.lock` is `.gitignore`d so PR CI didn't catch it. Pinned `specta` + `tauri-specta` to `=2.0.0-rc.20` and reverted `specta-typescript` to `=0.0.7`. Tightened the Dependabot ignore for `specta-typescript` from `>=0.0.12` to `>=0.0.8` since `0.0.8`–`0.0.11` all transitively pull the broken rc. Same merge-CI gap pattern as the earlier PR-#5/#7 incident.
- **Version drift** — `VERSION` and `package.json` were at `0.0.5` but `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` lagged at `0.0.4`. All four now sit at `0.1.0`.

### Dependencies

- **`react-diff-view ^3.3.3`** + transitive `gitdiff-parser` — unified-diff parsing + split/inline rendering for the diff viewer (Plan 5).
- **`@testing-library/react ^16.3.2`** + **`@testing-library/jest-dom ^6.9.1`** (devDependencies) — needed for the new component-level Vitest specs that came with each of the three plans.
- **`tauri-plugin-clipboard-manager = "2"`** (Rust) + **`@tauri-apps/plugin-clipboard-manager ^2.3.2`** (JS) — system clipboard access for the Copy PR Report flow (Plan 7).

### Notes

- **Reconcile-on-exit is frontend-driven**, not Rust-side. The Plan 5 architecture text described extending Plan 4's task-exit reaper, but the actual wiring lives in `TerminalView`'s `useEffect` on `exitCode`. Functional implication: if the user navigates away from the Agent Workspace panel before the engine exits, the auto-transition won't fire. The Rust-side reaper version is a follow-up if that becomes a problem in practice.
- **Verification persistence is finalized-only** — there's no streaming of live verification output to the UI yet. The user sees a spinner while a run executes, then the status pills appear when it completes. v0.2 will likely stream output similar to the agent terminal.
- **`gh` integration is non-draft only in the UI** — the `CreatePrRequest.draft` field exists on the wire and `create_pr` honors it, but the button passes `draft: false`. A toggle is a v0.2 nicety.
- **Plan 7 tests skip `$PATH` mutation entirely.** The plan's three `detect()`-via-fake-gh tests would have raced on the global `$PATH` env var under cargo's default parallel runner. Refactored: extract `which_in(name, path_var)` so PATH is an argument, and test `parse_account` directly against realistic `gh auth status` output. Net 7 deterministic unit tests instead of 3 racey integration ones.

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
