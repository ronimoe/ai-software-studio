# Auto-Dispatch Queue — Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A background worker that drains a FIFO queue of human-queued tasks, driving each from `Queued` → worktree → agent → verification → draft PR, sequentially, claude-code only, with retry-once and verification-gating.

**Architecture:** In-process Tokio worker (`DispatchWorker`) spawned at app startup, reusing existing services (`worktree_lifecycle`, `ProcessRunner`, `VerificationService`, `report::render`, `github`). Two new seams — `AgentLauncher` and `PrPublisher` — keep `run_pipeline` unit-testable without a real `claude`/`gh`. A new `ProcessRunner::wait_for_exit` gives Rust a way to await agent completion (also the Rust-side reconcile the spec calls for).

**Tech Stack:** Rust, Tokio, sqlx (SQLite), tauri-specta, dashmap.

**Spec:** `docs/superpowers/specs/2026-05-31-auto-dispatch-queue-design.md`

**Scope:** Backend only. UI (Queue button, dispatch control, hooks, mocks) is a sibling follow-up plan.

---

## File structure

- Create: `src-tauri/migrations/20260104000000_dispatch_queue.sql` — `queued_at` column + index.
- Modify: `src-tauri/src/models.rs` — `TaskStatus::Queued`, `Task.queued_at`.
- Modify: `src-tauri/src/tasks/repository.rs` — `queued_at` mapping; `enqueue`/`dequeue`/`next_queued`/`count_queued`/`ids_in_statuses`.
- Modify: `src-tauri/src/tasks/mod.rs` — `TaskService` wrappers.
- Modify: `src-tauri/src/process/mod.rs` — `ExitInfo`, `wait_for_exit`, stop marker.
- Modify: `src-tauri/src/engines/adapters/claude_code.rs` — make `build_prompt` `pub(crate)`.
- Create: `src-tauri/src/dispatch/mod.rs` — module root, `DispatchStatus`, `DispatchHandle`, `DispatchEvent`, autorun helpers.
- Create: `src-tauri/src/dispatch/seams.rs` — `AgentLauncher`/`ClaudeAgentLauncher`, `PrPublisher`/`GhPublisher`.
- Create: `src-tauri/src/dispatch/worker.rs` — `DispatchWorker`, `run_pipeline`, retry helpers.
- Create: `src-tauri/src/dispatch/sweep.rs` — `reconcile_orphans`.
- Create: `src-tauri/src/dispatch/worker_tests.rs` — integration tests.
- Create: `src-tauri/src/commands/dispatch.rs` — 5 commands.
- Modify: `src-tauri/src/commands/mod.rs` — `pub mod dispatch;`
- Modify: `src-tauri/src/state.rs` — `AppState.dispatch`.
- Modify: `src-tauri/src/lib.rs` — register commands + event, spawn worker, run sweep, mirror in export test.

---

## Task 1: Migration + `Queued` status + `queued_at` field

**Files:**
- Create: `src-tauri/migrations/20260104000000_dispatch_queue.sql`
- Modify: `src-tauri/src/models.rs:15-29` (enum), `src-tauri/src/models.rs:37-52` (struct)
- Modify: `src-tauri/src/tasks/repository.rs` (serialize/parse + `get` mapping)

- [ ] **Step 1: Write the migration**

Create `src-tauri/migrations/20260104000000_dispatch_queue.sql`:

```sql
ALTER TABLE tasks ADD COLUMN queued_at TEXT;
CREATE INDEX IF NOT EXISTS idx_tasks_queued ON tasks(queued_at);
```

- [ ] **Step 2: Add `Queued` to `TaskStatus`**

In `src-tauri/src/models.rs`, add `Queued` after `Draft`:

```rust
pub enum TaskStatus {
    Draft,
    Queued,
    WorktreeCreated,
    Running,
    NeedsInput,
    VerificationRunning,
    ReviewReady,
    Approved,
    PrPrepared,
    Done,
    ChangesRequested,
    Rejected,
    Failed,
    Stopped,
}
```

- [ ] **Step 3: Add `queued_at` to `Task`**

In `src-tauri/src/models.rs` `Task` struct, add after `created_at`:

```rust
    pub created_at: String,
    pub queued_at: Option<String>,
}
```

- [ ] **Step 4: Map status + column in the repository**

In `src-tauri/src/tasks/repository.rs`:

In `serialize_status`, add `TaskStatus::Queued => "queued",` (after `Draft`).
In `parse_status`, add `"queued" => TaskStatus::Queued,` (after `"draft"`).

Update `get`'s SELECT + row tuple to include `queued_at` (becomes the 13th column):

```rust
        let row: Option<(String, String, String, String, String, String, Option<String>, String, String, Option<String>, Option<String>, String, Option<String>)> = sqlx::query_as(
            "SELECT id, project_id, title, description, out_of_scope, files_to_touch_hint,
                    selected_engine, status, risk, branch_name, worktree_path, created_at, queued_at
             FROM tasks WHERE id = ?",
        )
```

And in the returned `Task { ... }`, add:

```rust
            created_at: row.11,
            queued_at: row.12,
```

- [ ] **Step 5: Add `queued_at: None` to the other `Task { .. }` literals**

The new field breaks every existing `Task { .. }` literal. Add `queued_at: None,` to each (after `created_at`):
- `src-tauri/src/fixtures.rs` — 5 literals (lines ~20, ~40, ~59, ~77, ~93)
- `src-tauri/src/core/worktree_context_tests.rs`
- `src-tauri/src/tasks/brief_tests.rs`
- `src-tauri/src/commands/pr/report_tests.rs`

Then run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: compiles. (Confirm none were missed with `rg "Task \{" src-tauri/src`.)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/migrations/20260104000000_dispatch_queue.sql src-tauri/src/models.rs src-tauri/src/tasks/repository.rs
git commit -m "feat(dispatch): add Queued status + queued_at column"
```

---

## Task 2: Queue repository methods + service wrappers

**Files:**
- Modify: `src-tauri/src/tasks/repository.rs`
- Modify: `src-tauri/src/tasks/mod.rs`
- Test: `src-tauri/src/tasks/repository_tests.rs`

- [ ] **Step 1: Write the failing test**

Append to `src-tauri/src/tasks/repository_tests.rs`. It reuses that file's existing `seed_project` and `request` helpers:

```rust
#[tokio::test]
async fn enqueue_is_fifo_and_dequeue_resets() {
    let db = Db::test_pool().await.expect("db");
    seed_project(&db, "proj-1").await;
    let repo = TaskRepository::new(db);
    let a = repo.insert(&request("proj-1", "A")).await.expect("a");
    let b = repo.insert(&request("proj-1", "B")).await.expect("b");

    repo.enqueue(&a.id).await.unwrap();
    repo.enqueue(&b.id).await.unwrap();

    // Oldest queued_at first; rowid ASC breaks ms-resolution ties deterministically.
    let next = repo.next_queued().await.unwrap().expect("a queued task");
    assert_eq!(next.id, a.id);
    assert_eq!(next.status, TaskStatus::Queued);
    assert!(next.queued_at.is_some());
    assert_eq!(repo.count_queued().await.unwrap(), 2);

    repo.dequeue(&a.id).await.unwrap();
    let after = repo.get(&a.id).await.unwrap();
    assert_eq!(after.status, TaskStatus::Draft);
    assert!(after.queued_at.is_none());
    assert_eq!(repo.count_queued().await.unwrap(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml enqueue_sets_status -- --nocapture`
Expected: FAIL — `no method named enqueue`.

- [ ] **Step 3: Implement repository methods**

Add to `impl TaskRepository` in `src-tauri/src/tasks/repository.rs`:

```rust
    pub async fn enqueue(&self, task_id: &str) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE tasks SET status = 'queued', queued_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?",
        )
        .bind(task_id)
        .execute(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("enqueue: {e}")))?;
        Ok(())
    }

    pub async fn dequeue(&self, task_id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE tasks SET status = 'draft', queued_at = NULL WHERE id = ?")
            .bind(task_id)
            .execute(&self.db.pool)
            .await
            .map_err(|e| AppError::internal(format!("dequeue: {e}")))?;
        Ok(())
    }

    pub async fn next_queued(&self) -> Result<Option<Task>, AppError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM tasks WHERE status = 'queued' ORDER BY queued_at ASC, rowid ASC LIMIT 1",
        )
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError::internal(format!("next_queued: {e}")))?;
        match row {
            Some((id,)) => Ok(Some(self.get(&id).await?)),
            None => Ok(None),
        }
    }

    pub async fn count_queued(&self) -> Result<u32, AppError> {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE status = 'queued'")
            .fetch_one(&self.db.pool)
            .await
            .map_err(|e| AppError::internal(format!("count_queued: {e}")))?;
        Ok(n as u32)
    }

    pub async fn ids_in_statuses(&self, statuses: &[&str]) -> Result<Vec<String>, AppError> {
        let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("SELECT id FROM tasks WHERE status IN ({placeholders})");
        let mut q = sqlx::query_as::<_, (String,)>(&sql);
        for s in statuses {
            q = q.bind(*s);
        }
        let rows = q
            .fetch_all(&self.db.pool)
            .await
            .map_err(|e| AppError::internal(format!("ids_in_statuses: {e}")))?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
```

- [ ] **Step 4: Add `TaskService` wrappers**

Add to `impl TaskService` in `src-tauri/src/tasks/mod.rs`:

```rust
    pub async fn enqueue(&self, task_id: &str) -> Result<(), AppError> {
        self.repo.enqueue(task_id).await
    }
    pub async fn dequeue(&self, task_id: &str) -> Result<(), AppError> {
        self.repo.dequeue(task_id).await
    }
    pub async fn next_queued(&self) -> Result<Option<Task>, AppError> {
        self.repo.next_queued().await
    }
    pub async fn count_queued(&self) -> Result<u32, AppError> {
        self.repo.count_queued().await
    }
    pub async fn ids_in_statuses(&self, statuses: &[&str]) -> Result<Vec<String>, AppError> {
        self.repo.ids_in_statuses(statuses).await
    }
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml enqueue_sets_status`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tasks/
git commit -m "feat(dispatch): queue repository methods (enqueue/dequeue/next_queued)"
```

---

## Task 3: `ProcessRunner::wait_for_exit` + `ExitInfo`

**Files:**
- Modify: `src-tauri/src/process/mod.rs`
- Test: `src-tauri/src/process/tests.rs`

- [ ] **Step 1: Write the failing test**

Append to `src-tauri/src/process/tests.rs`:

```rust
#[tokio::test]
async fn wait_for_exit_reports_clean_exit() {
    let runner = ProcessRunner::new();
    runner
        .spawn("t-exit", "sh", &["-c", "exit 0"], &std::env::temp_dir())
        .await
        .unwrap();
    let info = runner.wait_for_exit("t-exit").await;
    assert_eq!(info.exit_code, Some(0));
    assert!(!info.stopped_by_user);
}

#[tokio::test]
async fn wait_for_exit_reports_user_stop() {
    let runner = ProcessRunner::new();
    runner
        .spawn("t-stop", "sh", &["-c", "sleep 10"], &std::env::temp_dir())
        .await
        .unwrap();
    runner.stop("t-stop").await.unwrap();
    let info = runner.wait_for_exit("t-stop").await;
    assert!(info.stopped_by_user, "stop() must mark stopped_by_user");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml wait_for_exit -- --nocapture`
Expected: FAIL — `no method named wait_for_exit`.

- [ ] **Step 3: Add `ExitInfo` + new fields**

In `src-tauri/src/process/mod.rs`, add the type near the top:

```rust
#[derive(Clone, Copy, Debug)]
pub struct ExitInfo {
    pub exit_code: Option<i32>,
    pub signaled: bool,
    pub stopped_by_user: bool,
}
```

Extend the struct + `new`:

```rust
pub struct ProcessRunner {
    handle: Mutex<Option<AppHandle>>,
    running: Arc<DashMap<String, Arc<Mutex<Child>>>>,
    exits: Arc<DashMap<String, tokio::sync::watch::Receiver<Option<ExitInfo>>>>,
    stop_requests: Arc<DashMap<String, ()>>,
}

impl ProcessRunner {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
            running: Arc::new(DashMap::new()),
            exits: Arc::new(DashMap::new()),
            stop_requests: Arc::new(DashMap::new()),
        }
    }
```

- [ ] **Step 4: Wire the completion channel in `spawn` + reaper**

In `spawn`, right after `running.insert(...)`, create the channel:

```rust
        running.insert(task_id_owned.clone(), child_arc.clone());
        let (exit_tx, exit_rx) = tokio::sync::watch::channel(None);
        self.exits.insert(task_id_owned.clone(), exit_rx);
```

Replace the reaper `tokio::spawn(async move { ... })` block with one that also resolves the channel and consumes the stop marker:

```rust
        let handle_for_reaper = handle_opt;
        let running_for_reaper = running.clone();
        let stop_requests = self.stop_requests.clone();
        let task_id_for_reaper = task_id_owned;
        tokio::spawn(async move {
            let mut guard = child_arc.lock().await;
            let exit = guard.wait().await;
            running_for_reaper.remove(&task_id_for_reaper);
            let stopped_by_user = stop_requests.remove(&task_id_for_reaper).is_some();
            let exit_code = exit.as_ref().ok().and_then(|s| s.code());
            let signaled = exit.as_ref().ok().map(|s| s.code().is_none()).unwrap_or(false);
            let _ = exit_tx.send(Some(ExitInfo { exit_code, signaled, stopped_by_user }));
            if let Some(h) = handle_for_reaper {
                let payload = TaskExit {
                    task_id: task_id_for_reaper.clone(),
                    exit_code,
                    signaled,
                };
                let _ = payload.emit(&h);
            }
        });
```

- [ ] **Step 5: Add `wait_for_exit` + stop marker**

Add to `impl ProcessRunner`:

```rust
    /// Await the agent's exit. Returns immediately if the process already exited.
    /// Returns a default (unknown) ExitInfo if the task was never spawned.
    pub async fn wait_for_exit(&self, task_id: &str) -> ExitInfo {
        let mut rx = match self.exits.get(task_id) {
            Some(r) => r.clone(),
            None => return ExitInfo { exit_code: None, signaled: false, stopped_by_user: false },
        };
        loop {
            if let Some(info) = *rx.borrow() {
                return info;
            }
            if rx.changed().await.is_err() {
                return ExitInfo { exit_code: None, signaled: false, stopped_by_user: false };
            }
        }
    }
```

In `stop`, mark the request as the very first line (before the early `return Ok(())`):

```rust
    pub async fn stop(&self, task_id: &str) -> Result<(), AppError> {
        self.stop_requests.insert(task_id.to_string(), ());
        let Some(entry) = self.running.get(task_id) else { return Ok(()); };
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml wait_for_exit`
Expected: PASS (both).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/process/
git commit -m "feat(dispatch): ProcessRunner::wait_for_exit + ExitInfo (Rust-side reaper)"
```

---

## Task 4: Seams — `AgentLauncher` + `PrPublisher`

**Files:**
- Create: `src-tauri/src/dispatch/mod.rs` (module stub for now)
- Create: `src-tauri/src/dispatch/seams.rs`
- Modify: `src-tauri/src/engines/adapters/claude_code.rs` (make `build_prompt` `pub(crate)`)
- Modify: `src-tauri/src/lib.rs` (`pub mod dispatch;`)

- [ ] **Step 1: Expose `build_prompt`**

In `src-tauri/src/engines/adapters/claude_code.rs`, change `fn build_prompt` to `pub(crate) fn build_prompt`.

- [ ] **Step 2: Create the module root**

Create `src-tauri/src/dispatch/mod.rs`:

```rust
pub mod seams;
```

Add `pub mod dispatch;` to the module list at the top of `src-tauri/src/lib.rs` (alphabetical, after `pub mod core;`... place near `pub mod db;`).

- [ ] **Step 3: Implement the seams**

Create `src-tauri/src/dispatch/seams.rs`:

```rust
use crate::{error::AppError, process::ProcessRunner};
use std::path::Path;

/// Resolves the command to spawn for a task's agent run. A seam so tests can
/// inject a fake short-lived process instead of the real `claude` binary.
pub trait AgentLauncher: Send + Sync {
    /// Returns `(program, args)` to spawn, or Err if the engine is unavailable.
    fn command(&self, task_id: &str) -> Result<(String, Vec<String>), AppError>;
}

pub struct ClaudeAgentLauncher;

impl AgentLauncher for ClaudeAgentLauncher {
    fn command(&self, task_id: &str) -> Result<(String, Vec<String>), AppError> {
        let claude = crate::engines::detection::detect_claude()?;
        let binary = claude
            .binary_path
            .ok_or_else(|| AppError::internal("claude binary not found on PATH"))?;
        let prompt = crate::engines::adapters::claude_code::build_prompt(task_id);
        Ok((binary, vec!["--print".to_string(), prompt]))
    }
}

/// Pushes a branch and opens a PR. A seam so tests don't shell out to real `gh`.
pub trait PrPublisher: Send + Sync {
    fn push_branch(&self, repo: &Path, branch: &str) -> Result<(), AppError>;
    fn create_pr(&self, repo: &Path, title: &str, body: &str, base: &str, draft: bool)
        -> Result<String, AppError>;
}

pub struct GhPublisher;

impl PrPublisher for GhPublisher {
    fn push_branch(&self, repo: &Path, branch: &str) -> Result<(), AppError> {
        crate::engines::github::push_branch(repo, branch)
    }
    fn create_pr(&self, repo: &Path, title: &str, body: &str, base: &str, draft: bool)
        -> Result<String, AppError> {
        crate::engines::github::create_pr(repo, title, body, base, draft)
    }
}

/// Spawn a task's agent through the given launcher + runner.
pub async fn launch_agent(
    launcher: &dyn AgentLauncher,
    runner: &ProcessRunner,
    task_id: &str,
    worktree: &Path,
) -> Result<(), AppError> {
    let (program, args) = launcher.command(task_id)?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    runner.spawn(task_id, &program, &arg_refs, &worktree.to_path_buf()).await
}
```

- [ ] **Step 4: Build to verify compilation**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: compiles.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/dispatch/ src-tauri/src/engines/adapters/claude_code.rs src-tauri/src/lib.rs
git commit -m "feat(dispatch): AgentLauncher + PrPublisher seams"
```

---

## Task 5: Dispatch types — status, handle, event, autorun persistence

**Files:**
- Modify: `src-tauri/src/dispatch/mod.rs`

- [ ] **Step 1: Add the types + handle + autorun helpers**

Replace `src-tauri/src/dispatch/mod.rs` with the following. **Only `seams` is declared here** — `worker`, `sweep`, and `worker_tests` are added by their own tasks as the files are created, so the crate compiles at every step:

```rust
pub mod seams;
// pub mod worker;   <- added in Task 6
// pub mod sweep;    <- added in Task 8
// #[cfg(test)] mod worker_tests;  <- added in Task 11

use crate::{db::Db, error::AppError};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri_specta::Event;
use tokio::sync::{Mutex, Notify};

/// Snapshot of the dispatcher for the UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DispatchStatus {
    pub running: bool,
    pub queued: u32,
    pub current_task: Option<String>,
}

/// Live narration of what the worker is doing. UI-only; durable state is the task status.
#[derive(Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct DispatchEvent {
    pub task_id: String,
    pub stage: String,
    pub outcome: String,
}

/// Shared control surface between commands and the worker.
#[derive(Clone)]
pub struct DispatchHandle {
    pub notify: Arc<Notify>,
    pub paused: Arc<AtomicBool>,
    pub current_task: Arc<Mutex<Option<String>>>,
}

impl DispatchHandle {
    pub fn new(paused: bool) -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            paused: Arc::new(AtomicBool::new(paused)),
            current_task: Arc::new(Mutex::new(None)),
        }
    }
    pub fn wake(&self) {
        self.notify.notify_one();
    }
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        self.wake();
    }
}

const AUTORUN_SCOPE: &str = "global";
const AUTORUN_KEY: &str = "dispatch.autorun";

/// Read the persisted autorun flag (defaults to true / running).
pub async fn get_autorun(db: &Db) -> Result<bool, AppError> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_settings WHERE scope = ? AND key = ?")
            .bind(AUTORUN_SCOPE)
            .bind(AUTORUN_KEY)
            .fetch_optional(&db.pool)
            .await
            .map_err(|e| AppError::internal(format!("get autorun: {e}")))?;
    Ok(row.map(|(v,)| v != "false").unwrap_or(true))
}

/// Persist the autorun flag.
pub async fn set_autorun(db: &Db, on: bool) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app_settings (scope, key, value) VALUES (?, ?, ?)
         ON CONFLICT(scope, key) DO UPDATE SET value = excluded.value",
    )
    .bind(AUTORUN_SCOPE)
    .bind(AUTORUN_KEY)
    .bind(if on { "true" } else { "false" })
    .execute(&db.pool)
    .await
    .map_err(|e| AppError::internal(format!("set autorun: {e}")))?;
    Ok(())
}
```

> Note: `app_settings(scope, key, value)` with `(scope, key)` PK already exists (migration `20260103000000_verification.sql`).

- [ ] **Step 2: Add `dispatch` to `AppState`**

In `src-tauri/src/state.rs`, add the import:

```rust
use crate::dispatch::{get_autorun, DispatchHandle};
```

Add the field to the struct (after `process`):

```rust
    pub process: Arc<ProcessRunner>,
    pub dispatch: DispatchHandle,
}
```

In `init`, replace `let db = Db::init().await?;` with:

```rust
        let db = Db::init().await?;
        let autorun = get_autorun(&db).await?;
        let dispatch = DispatchHandle::new(!autorun);
```

And add `dispatch,` to the returned struct literal.

- [ ] **Step 3: Build to verify compilation**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: compiles — `mod.rs` declares only the already-created `seams`, and `AppState` now carries `dispatch`.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/dispatch/mod.rs src-tauri/src/state.rs
git commit -m "feat(dispatch): DispatchHandle/Status/Event, autorun persistence, AppState wiring"
```

---

## Task 6: `DispatchWorker` struct, loop, and helpers

**Files:**
- Create: `src-tauri/src/dispatch/worker.rs`

- [ ] **Step 1: Declare the module, then create the worker skeleton + loop + helpers**

Add `pub mod worker;` to `src-tauri/src/dispatch/mod.rs` (replacing the `// pub mod worker;` comment). Then create `src-tauri/src/dispatch/worker.rs`:

```rust
use std::path::Path;
use std::sync::Arc;

use crate::{
    core::worktree_context::WorktreeContextService,
    db::Db,
    dispatch::{
        seams::{launch_agent, AgentLauncher, PrPublisher},
        DispatchEvent, DispatchHandle,
    },
    error::AppError,
    git::{
        worktree_paths::{branch_name, worktree_path},
        GitService,
    },
    models::{ChangedFile, Task, TaskStatus, VerificationStatus},
    process::ProcessRunner,
    projects::ProjectService,
    tasks::TaskService,
    verification::VerificationService,
};
use tauri::AppHandle;
use tauri_specta::Event;

enum AgentOutcome {
    Success,
    Errored,
    Stopped,
}

pub struct DispatchWorker {
    pub tasks: TaskService,
    pub projects: ProjectService,
    pub verification: VerificationService,
    pub git: GitService,
    pub worktree_context: WorktreeContextService,
    pub process: Arc<ProcessRunner>,
    pub agent: Arc<dyn AgentLauncher>,
    pub publisher: Arc<dyn PrPublisher>,
    pub handle: DispatchHandle,
    pub app: Option<AppHandle>,
}

impl DispatchWorker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Db,
        process: Arc<ProcessRunner>,
        agent: Arc<dyn AgentLauncher>,
        publisher: Arc<dyn PrPublisher>,
        handle: DispatchHandle,
        app: Option<AppHandle>,
    ) -> Self {
        Self {
            tasks: TaskService::new(db.clone()),
            projects: ProjectService::new(db.clone()),
            verification: VerificationService::new(db.clone()),
            git: GitService::new(),
            worktree_context: WorktreeContextService::new(),
            process,
            agent,
            publisher,
            handle,
            app,
        }
    }

    /// Long-lived loop. Pulls one queued task at a time, drives it to a terminal
    /// state, then pulls the next. Sleeps on `notify` when idle or paused.
    pub async fn run(self: Arc<Self>) {
        loop {
            if self.handle.is_paused() {
                self.handle.notify.notified().await;
                continue;
            }
            match self.tasks.next_queued().await {
                Ok(Some(task)) => {
                    *self.handle.current_task.lock().await = Some(task.id.clone());
                    self.run_pipeline(&task).await;
                    *self.handle.current_task.lock().await = None;
                }
                Ok(None) => self.handle.notify.notified().await,
                Err(_) => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
            }
        }
    }

    fn emit(&self, task_id: &str, stage: &str, outcome: &str) {
        if let Some(app) = &self.app {
            let _ = DispatchEvent {
                task_id: task_id.to_string(),
                stage: stage.to_string(),
                outcome: outcome.to_string(),
            }
            .emit(app);
        }
    }

    async fn set(&self, task_id: &str, status: TaskStatus) {
        let _ = self.tasks.update_status(task_id, status).await;
    }

    async fn git_status(&self, worktree: &Path) -> Result<Vec<ChangedFile>, AppError> {
        let p = worktree.to_path_buf();
        tokio::task::spawn_blocking(move || crate::git::status::status(&p))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))?
    }
}
```

- [ ] **Step 2: Build (will warn about unused helpers until Task 7)**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: compiles (dead-code warnings on `AgentOutcome`/`git_status`/`emit`/`set` are fine until Task 7).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/dispatch/worker.rs
git commit -m "feat(dispatch): DispatchWorker struct + run loop"
```

---

## Task 7: `run_pipeline` + retry helpers (the core)

**Files:**
- Modify: `src-tauri/src/dispatch/worker.rs`

- [ ] **Step 1: Add the pipeline + retry helpers**

Add these methods to `impl DispatchWorker` in `src-tauri/src/dispatch/worker.rs`:

```rust
    /// Drive one task from Queued to a terminal state. Never panics; all errors
    /// land the task in a resting status and the loop continues.
    /// `pub(crate)` so `worker_tests` (a sibling module) can drive it directly.
    pub(crate) async fn run_pipeline(&self, task: &Task) {
        // Codex guard (defensive — enqueue already rejects non-claude).
        let engine = task.selected_engine.as_deref().unwrap_or("claude-code");
        if engine != "claude-code" {
            self.emit(&task.id, "guard", "unsupported-engine");
            self.set(&task.id, TaskStatus::Failed).await;
            return;
        }

        // Stage 1: worktree.
        self.emit(&task.id, "worktree", "start");
        let project = match self.projects.get(&task.project_id).await {
            Ok(p) => p,
            Err(_) => return self.fail(&task.id, "worktree").await,
        };
        let branch = branch_name(&task.id);
        let dest = match worktree_path(&task.project_id, &task.id) {
            Ok(d) => d,
            Err(_) => return self.fail(&task.id, "worktree").await,
        };
        let repo = std::path::PathBuf::from(&project.path);
        // Capture the returned Task — it now carries branch_name + worktree_path,
        // which the PR stage needs. The original `task` arg is still None for those.
        let task = match crate::core::worktree_lifecycle::create_worktree_lifecycle(
            &self.git,
            &self.worktree_context,
            &self.tasks,
            task,
            &repo,
            &branch,
            &dest,
            &project.default_branch,
        )
        .await
        {
            Ok(t) => t,
            Err(_) => return self.fail(&task.id, "worktree").await,
        };

        // Stage 2: agent (one retry).
        self.emit(&task.id, "agent", "start");
        match self.run_agent_with_retry(&task.id, &dest).await {
            AgentOutcome::Success => {}
            AgentOutcome::Stopped => {
                self.emit(&task.id, "agent", "stopped");
                return self.set(&task.id, TaskStatus::Stopped).await;
            }
            AgentOutcome::Errored => return self.fail(&task.id, "agent").await,
        }

        // Stage 3: reconcile.
        let changed = match self.git_status(&dest).await {
            Ok(c) => c,
            Err(_) => return self.set(&task.id, TaskStatus::Stopped).await,
        };
        if changed.is_empty() {
            self.emit(&task.id, "reconcile", "no-changes");
            return self.set(&task.id, TaskStatus::Stopped).await;
        }

        // Stage 4: verification (one re-run). Failure stops at ReviewReady, no PR.
        self.emit(&task.id, "verify", "start");
        self.set(&task.id, TaskStatus::VerificationRunning).await;
        if !self.verify_with_retry(&task.project_id, &task.id, &dest).await {
            self.emit(&task.id, "verify", "failed");
            return self.set(&task.id, TaskStatus::ReviewReady).await;
        }

        // Stage 5: draft PR (one retry). Failure stops at ReviewReady (work is good).
        self.emit(&task.id, "pr", "start");
        if !self.publish_pr_with_retry(&task, &project.default_branch, &dest).await {
            self.emit(&task.id, "pr", "failed");
            return self.set(&task.id, TaskStatus::ReviewReady).await;
        }
        self.set(&task.id, TaskStatus::PrPrepared).await;
        self.emit(&task.id, "pr", "ok");
    }

    async fn fail(&self, task_id: &str, stage: &str) {
        self.emit(task_id, stage, "failed");
        self.set(task_id, TaskStatus::Failed).await;
    }

    async fn run_agent(&self, task_id: &str, worktree: &Path) -> AgentOutcome {
        self.set(task_id, TaskStatus::Running).await;
        if launch_agent(self.agent.as_ref(), &self.process, task_id, worktree)
            .await
            .is_err()
        {
            return AgentOutcome::Errored;
        }
        let info = self.process.wait_for_exit(task_id).await;
        if info.stopped_by_user {
            AgentOutcome::Stopped
        } else if info.exit_code == Some(0) && !info.signaled {
            AgentOutcome::Success
        } else {
            AgentOutcome::Errored
        }
    }

    async fn run_agent_with_retry(&self, task_id: &str, worktree: &Path) -> AgentOutcome {
        match self.run_agent(task_id, worktree).await {
            AgentOutcome::Errored => {
                self.emit(task_id, "agent", "retry");
                self.run_agent(task_id, worktree).await
            }
            other => other,
        }
    }

    async fn verify_once(&self, project_id: &str, task_id: &str, worktree: &Path) -> bool {
        match self.verification.run_for_task(project_id, task_id, worktree).await {
            Ok(run) => !run.checks.iter().any(|c| c.status == VerificationStatus::Failed),
            Err(_) => false,
        }
    }

    async fn verify_with_retry(&self, project_id: &str, task_id: &str, worktree: &Path) -> bool {
        if self.verify_once(project_id, task_id, worktree).await {
            return true;
        }
        self.emit(task_id, "verify", "retry");
        self.verify_once(project_id, task_id, worktree).await
    }

    async fn publish_once(&self, task: &Task, base: &str, worktree: &Path) -> Result<(), AppError> {
        let branch = task
            .branch_name
            .clone()
            .ok_or_else(|| AppError::invalid_arg("task has no branch"))?;
        let runs = self.verification.list_for_task(&task.id).await?;
        let latest = runs.first().cloned();
        let changed = self.git_status(worktree).await?;
        let body = crate::commands::pr::report::render(task, &changed, latest.as_ref());

        let publisher = self.publisher.clone();
        let repo = worktree.to_path_buf();
        let branch_c = branch.clone();
        tokio::task::spawn_blocking(move || publisher.push_branch(&repo, &branch_c))
            .await
            .map_err(|e| AppError::internal(format!("join: {e}")))??;

        let publisher = self.publisher.clone();
        let repo = worktree.to_path_buf();
        let title = task.title.clone();
        let base_c = base.to_string();
        tokio::task::spawn_blocking(move || {
            publisher.create_pr(&repo, &title, &body, &base_c, true)
        })
        .await
        .map_err(|e| AppError::internal(format!("join: {e}")))??;
        Ok(())
    }

    async fn publish_pr_with_retry(&self, task: &Task, base: &str, worktree: &Path) -> bool {
        if self.publish_once(task, base, worktree).await.is_ok() {
            return true;
        }
        self.emit(&task.id, "pr", "retry");
        self.publish_once(task, base, worktree).await.is_ok()
    }
```

- [ ] **Step 2: Build**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: compiles (the integration test in Task 11 exercises this).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/dispatch/worker.rs
git commit -m "feat(dispatch): run_pipeline + agent/verify/pr retry"
```

---

## Task 8: Startup sweep (`reconcile_orphans`)

**Files:**
- Create: `src-tauri/src/dispatch/sweep.rs`
- Test: covered by `worker_tests.rs` in Task 11 (a seeded `Running` task)

- [ ] **Step 1: Declare the module, then implement the sweep**

Add `pub mod sweep;` to `src-tauri/src/dispatch/mod.rs` (replacing the `// pub mod sweep;` comment). Then create `src-tauri/src/dispatch/sweep.rs`:

```rust
use crate::{error::AppError, models::TaskStatus, tasks::TaskService};
use std::path::PathBuf;

/// On startup, reconcile tasks left mid-flight by a previous (crashed/quit)
/// session. Running/VerificationRunning → ReviewReady (changes) or Stopped
/// (clean). Never auto-retries; Queued tasks are untouched (the worker drains them).
pub async fn reconcile_orphans(tasks: &TaskService) -> Result<(), AppError> {
    let ids = tasks
        .ids_in_statuses(&["running", "verificationRunning"])
        .await?;
    for id in ids {
        let task = tasks.get(&id).await?;
        let new_status = match &task.worktree_path {
            Some(wt) => {
                let p = PathBuf::from(wt);
                let changed = tokio::task::spawn_blocking(move || crate::git::status::status(&p))
                    .await
                    .map_err(|e| AppError::internal(format!("join: {e}")))?
                    .unwrap_or_default();
                if changed.is_empty() {
                    TaskStatus::Stopped
                } else {
                    TaskStatus::ReviewReady
                }
            }
            None => TaskStatus::Stopped,
        };
        tasks.update_status(&id, new_status).await?;
    }
    Ok(())
}
```

- [ ] **Step 2: Build**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5`
Expected: compiles.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/dispatch/sweep.rs
git commit -m "feat(dispatch): startup sweep reconciles orphaned tasks"
```

---

## Task 9: Tauri commands

**Files:**
- Create: `src-tauri/src/commands/dispatch.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Add the module**

Add `pub mod dispatch;` to `src-tauri/src/commands/mod.rs` (alphabetical, after `pub mod diffs;`).

- [ ] **Step 2: Implement the commands**

Create `src-tauri/src/commands/dispatch.rs`:

```rust
use crate::{
    dispatch::{set_autorun, DispatchStatus},
    error::AppError,
    models::{Task, TaskStatus},
    state::AppState,
};
use tauri::State;

/// Pure guard for queue eligibility — extracted so it's unit-testable without `AppState`.
pub(crate) fn enqueue_eligibility(status: TaskStatus, engine: Option<&str>) -> Result<(), AppError> {
    match status {
        TaskStatus::Draft
        | TaskStatus::Stopped
        | TaskStatus::Failed
        | TaskStatus::ChangesRequested => {}
        other => {
            return Err(AppError::invalid_arg(format!(
                "cannot queue task in status {other:?}"
            )))
        }
    }
    let engine = engine.unwrap_or("claude-code");
    if engine != "claude-code" {
        return Err(AppError::invalid_arg(format!(
            "engine '{engine}' has no adapter yet; only claude-code is dispatchable"
        )));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn enqueue_task(state: State<'_, AppState>, task_id: String) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    enqueue_eligibility(task.status, task.selected_engine.as_deref())?;
    state.tasks.enqueue(&task_id).await?;
    state.dispatch.wake();
    state.tasks.get(&task_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn dequeue_task(state: State<'_, AppState>, task_id: String) -> Result<Task, AppError> {
    let task = state.tasks.get(&task_id).await?;
    if task.status != TaskStatus::Queued {
        return Err(AppError::invalid_arg("task is not queued"));
    }
    state.tasks.dequeue(&task_id).await?;
    state.tasks.get(&task_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_dispatch_status(state: State<'_, AppState>) -> Result<DispatchStatus, AppError> {
    let queued = state.tasks.count_queued().await?;
    let current_task = state.dispatch.current_task.lock().await.clone();
    Ok(DispatchStatus {
        running: !state.dispatch.is_paused(),
        queued,
        current_task,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn pause_dispatch(state: State<'_, AppState>) -> Result<(), AppError> {
    state.dispatch.pause();
    set_autorun(&state.db, false).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn resume_dispatch(state: State<'_, AppState>) -> Result<(), AppError> {
    state.dispatch.resume();
    set_autorun(&state.db, true).await?;
    Ok(())
}
```

- [ ] **Step 3: Add the guard test**

Append to `src-tauri/src/commands/dispatch.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_draft_stopped_and_null_engine() {
        assert!(enqueue_eligibility(TaskStatus::Draft, Some("claude-code")).is_ok());
        assert!(enqueue_eligibility(TaskStatus::Draft, None).is_ok()); // null → claude-code
        assert!(enqueue_eligibility(TaskStatus::Stopped, Some("claude-code")).is_ok());
        assert!(enqueue_eligibility(TaskStatus::ChangesRequested, Some("claude-code")).is_ok());
    }

    #[test]
    fn rejects_codex_engine() {
        assert!(enqueue_eligibility(TaskStatus::Draft, Some("codex-cli")).is_err());
    }

    #[test]
    fn rejects_non_requeuable_status() {
        assert!(enqueue_eligibility(TaskStatus::Running, Some("claude-code")).is_err());
        assert!(enqueue_eligibility(TaskStatus::Queued, Some("claude-code")).is_err());
    }
}
```

- [ ] **Step 4: Build + run the guard test**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands::dispatch::tests`
Expected: 3 pass. (`AppState.dispatch` was added in Task 5; commands are registered in `lib.rs` in Task 10.)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/
git commit -m "feat(dispatch): enqueue/dequeue/status/pause/resume commands + guard test"
```

---

## Task 10: Wiring — worker spawn, command/event registration

**Files:**
- Modify: `src-tauri/src/lib.rs`

(`AppState.dispatch` was already added in Task 5.)

- [ ] **Step 1: Register commands + event + spawn the worker**

In `src-tauri/src/lib.rs`, add the five commands to **both** `collect_commands![...]` blocks (the `run()` one and the `export_bindings_test` one), after `commands::pr::create_pr,`:

```rust
            commands::dispatch::enqueue_task,
            commands::dispatch::dequeue_task,
            commands::dispatch::get_dispatch_status,
            commands::dispatch::pause_dispatch,
            commands::dispatch::resume_dispatch,
```

Add the event to **both** `collect_events![...]` blocks, after `crate::process::TaskExit,`:

```rust
            crate::dispatch::DispatchEvent,
```

In the `.setup(...)` closure, replace the `Ok(state) => { ... }` arm with:

```rust
                    Ok(state) => {
                        state.process.set_handle(handle.clone()).await;
                        let _ = crate::dispatch::sweep::reconcile_orphans(&state.tasks).await;
                        let worker = std::sync::Arc::new(crate::dispatch::worker::DispatchWorker::new(
                            state.db.clone(),
                            state.process.clone(),
                            std::sync::Arc::new(crate::dispatch::seams::ClaudeAgentLauncher),
                            std::sync::Arc::new(crate::dispatch::seams::GhPublisher),
                            state.dispatch.clone(),
                            Some(handle.clone()),
                        ));
                        tauri::async_runtime::spawn(worker.run());
                        handle.manage(state);
                    }
```

- [ ] **Step 2: Build + regenerate bindings**

Run: `cargo build --manifest-path src-tauri/Cargo.toml && pnpm gen:bindings`
Expected: compiles; `lib/bindings.ts` regenerates with the 5 new commands + `DispatchEvent`.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(dispatch): wire worker + commands + event into app startup"
```

---

## Task 11: Integration tests for `run_pipeline`

**Files:**
- Create: `src-tauri/src/dispatch/worker_tests.rs`

- [ ] **Step 1: Declare the test module, then write the tests**

Add `#[cfg(test)] mod worker_tests;` to `src-tauri/src/dispatch/mod.rs` (replacing the `// #[cfg(test)] mod worker_tests;` comment). Then create `src-tauri/src/dispatch/worker_tests.rs`. These build a real temp git repo + in-memory DB, a fake agent (a shell script that optionally writes a file), and a fake publisher, then assert the terminal status.

```rust
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::{
    db::Db,
    dispatch::{
        seams::{AgentLauncher, PrPublisher},
        worker::DispatchWorker,
        DispatchHandle,
    },
    error::AppError,
    models::{CreateTaskRequest, TaskStatus, VerificationSettings},
    process::ProcessRunner,
};

// --- Fakes -----------------------------------------------------------------

/// Agent that runs `sh -c <script>` in the worktree — e.g. writes a file (changes)
/// or does nothing (no changes), and exits 0 or nonzero.
struct FakeAgent {
    script: String,
}
impl AgentLauncher for FakeAgent {
    fn command(&self, _task_id: &str) -> Result<(String, Vec<String>), AppError> {
        Ok(("sh".to_string(), vec!["-c".to_string(), self.script.clone()]))
    }
}

#[derive(Default, Clone)]
struct FakePublisher {
    calls: Arc<Mutex<u32>>,
}
impl PrPublisher for FakePublisher {
    fn push_branch(&self, _repo: &Path, _branch: &str) -> Result<(), AppError> {
        Ok(())
    }
    fn create_pr(&self, _r: &Path, _t: &str, _b: &str, _base: &str, _draft: bool)
        -> Result<String, AppError> {
        *self.calls.lock().unwrap() += 1;
        Ok("https://github.com/x/y/pull/1".to_string())
    }
}

// --- Harness ---------------------------------------------------------------

/// Creates an in-memory DB, a real git repo on disk registered as a project,
/// and a draft task. Returns (db, project_id, task_id, repo_dir).
async fn seed(db: &Db) -> (String, String) {
    // init a real git repo so worktree_lifecycle works
    let dir = tempfile::TempDir::new().unwrap().keep();
    run_git(&dir, &["init", "-q", "-b", "main"]);
    std::fs::write(dir.join("README.md"), "x").unwrap();
    run_git(&dir, &["add", "."]);
    run_git(&dir, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"]);

    let project_id = format!("proj-{}", uuid::Uuid::new_v4());
    sqlx::query("INSERT INTO projects (id, name, path, default_branch) VALUES (?, 'r', ?, 'main')")
        .bind(&project_id)
        .bind(dir.to_string_lossy().as_ref())
        .execute(&db.pool)
        .await
        .unwrap();

    let tasks = crate::tasks::TaskService::new(db.clone());
    let task = tasks
        .create(&CreateTaskRequest {
            project_id: project_id.clone(),
            title: "t".into(),
            description: "d".into(),
            out_of_scope: "".into(),
            files_to_touch_hint: "".into(),
            acceptance_criteria: vec!["ac".into()],
            constraints: vec![],
            selected_engine: Some("claude-code".into()),
        })
        .await
        .unwrap();
    tasks.enqueue(&task.id).await.unwrap();
    (project_id, task.id)
}

fn run_git(dir: &Path, args: &[&str]) {
    std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
}

fn worker(db: &Db, agent: FakeAgent, publisher: FakePublisher) -> Arc<DispatchWorker> {
    Arc::new(DispatchWorker::new(
        db.clone(),
        Arc::new(ProcessRunner::new()),
        Arc::new(agent),
        Arc::new(publisher),
        DispatchHandle::new(false),
        None,
    ))
}

// --- Tests -----------------------------------------------------------------

#[tokio::test]
async fn pipeline_reaches_pr_prepared_when_agent_changes_and_verification_passes() {
    let db = Db::test_pool().await.unwrap();
    let (project_id, task_id) = seed(&db).await;
    // verification: all pass
    crate::verification::VerificationService::new(db.clone())
        .set_settings(&project_id, &all_true_settings())
        .await
        .unwrap();

    let pub_fake = FakePublisher::default();
    let w = worker(&db, FakeAgent { script: "echo hi > new.txt".into() }, pub_fake.clone());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;

    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::PrPrepared);
    assert_eq!(*pub_fake.calls.lock().unwrap(), 1);
}

#[tokio::test]
async fn pipeline_stops_review_ready_when_verification_fails() {
    let db = Db::test_pool().await.unwrap();
    let (project_id, task_id) = seed(&db).await;
    crate::verification::VerificationService::new(db.clone())
        .set_settings(&project_id, &one_failing_setting())
        .await
        .unwrap();

    let pub_fake = FakePublisher::default();
    let w = worker(&db, FakeAgent { script: "echo hi > new.txt".into() }, pub_fake.clone());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;

    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::ReviewReady);
    assert_eq!(*pub_fake.calls.lock().unwrap(), 0, "no PR on red verification");
}

#[tokio::test]
async fn pipeline_stops_when_agent_makes_no_changes() {
    let db = Db::test_pool().await.unwrap();
    let (_project_id, task_id) = seed(&db).await;
    let w = worker(&db, FakeAgent { script: "true".into() }, FakePublisher::default());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;
    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::Stopped);
}

#[tokio::test]
async fn pipeline_fails_after_agent_errors_twice() {
    let db = Db::test_pool().await.unwrap();
    let (_project_id, task_id) = seed(&db).await;
    let w = worker(&db, FakeAgent { script: "exit 1".into() }, FakePublisher::default());
    let task = w.tasks.get(&task_id).await.unwrap();
    w.run_pipeline(&task).await;
    assert_eq!(w.tasks.get(&task_id).await.unwrap().status, TaskStatus::Failed);
}

fn all_true_settings() -> VerificationSettings {
    VerificationSettings {
        install: Some("true".into()),
        typecheck: Some("true".into()),
        lint: Some("true".into()),
        test: Some("true".into()),
        build: Some("true".into()),
    }
}

fn one_failing_setting() -> VerificationSettings {
    VerificationSettings {
        install: Some("true".into()),
        typecheck: Some("true".into()),
        lint: Some("true".into()),
        test: Some("false".into()), // exits nonzero → Failed check
        build: Some("true".into()),
    }
}
```

> `run_pipeline` is `pub(crate)` (Task 7), so `worker_tests` (a sibling module under `dispatch/`) can call `w.run_pipeline(...)` directly. The retry helpers stay private — they're exercised through `run_pipeline`.

- [ ] **Step 2: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml dispatch::worker_tests -- --test-threads=1`
Expected: 4 pass. (Single-threaded because each spins a real git repo + processes; avoids PATH/CWD contention.)

- [ ] **Step 3: Full suite + lint gate**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib && pnpm gen:bindings && pnpm typecheck`
Expected: all green; bindings regenerate cleanly.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/dispatch/worker_tests.rs
git commit -m "test(dispatch): run_pipeline integration tests (PrPrepared/ReviewReady/Stopped/Failed)"
```

---

## Done criteria

- `cargo test --lib` green, including the 4 `worker_tests` + `wait_for_exit` + queue-repo tests.
- `pnpm gen:bindings && pnpm typecheck` clean — `lib/bindings.ts` carries the 5 commands + `DispatchEvent`.
- Manually: enqueue a `claude-code` task → it auto-runs to a draft PR (or `ReviewReady` on red verification); a `codex-cli` task is rejected at enqueue; pausing stops new pulls; quitting mid-run then relaunching reconciles the orphan to `ReviewReady`/`Stopped`.

## Follow-up (sibling plan — UI)

`features/dispatch/*` hooks (`useEnqueueTask`, `useDequeueTask`, `useDispatchStatus`, `usePause/ResumeDispatch`), the Task Board "Queue" action + `queued` badge + dispatch control, and `lib/tauri.ts` mocks for the 5 commands.
