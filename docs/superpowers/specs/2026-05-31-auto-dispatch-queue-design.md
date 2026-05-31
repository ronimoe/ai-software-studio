# Auto-Dispatch Queue — Design

**Date:** 2026-05-31
**Status:** Approved (design); pending implementation plan
**Topic:** A background worker that drains a queue of human-approved tasks, driving each from intake all the way to a draft PR — hands-free — while preserving the human-approval boundary.

## Problem

Today the agent loop is fully human-gated, one click per milestone: **Create worktree → Start → (review) → Run verification → Create PR**. There is no mechanism to hand the app a backlog and have it work through it. This spec adds that: a **queue + worker** that auto-executes tasks the human has explicitly queued.

## Locked decisions

| Dimension | Decision |
|---|---|
| Core model | **Queue + worker** — a background dispatcher drains a FIFO queue (full batch autonomy) |
| Autonomy boundary | **Full pipeline to draft PR**, gated so verification failure never PRs |
| Concurrency | **Sequential (1)** — one task to a terminal state before the next is pulled |
| Enqueue gate | **Explicit "Queue" action** — entering the queue IS the human approval of intent |
| Engine scope | **claude-code only for now** — `codex-cli` tasks rejected with a clear "no adapter yet" reason |
| Failure policy | **Retry once, then stop**; verification failure never auto-PRs; failures don't block the rest of the queue |

## Non-goals (v1)

- Parallel execution (>1 concurrent). The worker is structured so a `Semaphore`-bounded `JoinSet` could raise it later, but it is not built now.
- Auto-fixing red verification (a fix-loop that re-runs the agent against failing tests). "Retry once" on verification re-runs *verification* only, to absorb flakes.
- A Codex adapter. `codex-cli` tasks are blocked from the queue.
- `NeedsInput` mid-run handling. `claude --print` is one-shot and cannot ask mid-run, so the worker treats agent exit as terminal. The `NeedsInput` path is reserved but unused.
- A separate audit/history table. Durable evidence is the already-persisted task status, `VerificationRun`, and PR URL.

## Architecture (Approach A — in-process Tokio worker)

A single long-lived async task spawned once at app startup (in `lib.rs` `setup`, after `AppState::init`). It reuses every existing service rather than reimplementing the pipeline: `worktree_lifecycle`, `ClaudeCodeAdapter`, `ProcessRunner`, `VerificationService`, and the PR logic.

Rejected alternatives:
- **B. Frontend-driven orchestrator** — only runs while the window/panel is mounted; inherits the "navigate away and it stalls" fragility; puts batch-automation logic in the client.
- **C. Event-driven reactor** — event-bus machinery and fuzzier ownership of transitions/retry; overkill for sequential-1.

### State machine & queue model

New status: **`TaskStatus::Queued`** (between `Draft` and `WorktreeCreated`).

New column: **`queued_at TEXT`** (nullable) on `tasks`, via migration `20260104000000_dispatch_queue.sql`. The queue is **derived, not a separate table**:

> the queue = tasks where `status = Queued`, ordered by `queued_at ASC` (FIFO).

Reuses the existing `idx_tasks_status` index.

**Enqueue gate (human intent boundary):**
- `enqueue_task(task_id)` — allowed from `Draft`, `Stopped`, `Failed`, `ChangesRequested` (re-queue supported). Guards: engine must be `claude-code` (or `null` → defaults to `claude-code`); a `codex-cli` task is rejected with a clear "no adapter yet" error. Sets `status = Queued`, `queued_at = now`.
- `dequeue_task(task_id)` — pulls a still-`Queued` task back to `Draft`.

**Per-task pipeline (each transition reuses an existing service/command):**

```
Queued
  → WorktreeCreated      (worktree_lifecycle)
  → Running              (ClaudeCodeAdapter::start)
  → [agent exits] reconcile (git::status):
       no changes  → Stopped            (done, nothing to PR)
       changes     → VerificationRunning (VerificationService)
  → verification result:
       any check FAILED → ReviewReady    (STOP — no PR, human triages)
       all clean        → push + draft PR → PrPrepared (done — human merges)
```

Worker-terminal states: `PrPrepared` (success), `ReviewReady` (verification red / PR step failed), `Stopped` (no changes / human Stop), `Failed` (agent errored twice / setup error). After any terminal state, the worker pulls the next `Queued` task.

### The worker loop

`DispatchWorker` holds references to the services plus two control primitives: a `tokio::sync::Notify` (wake signal) and a `tokio::sync::watch<bool>` (run/pause flag — the global "Auto-run" toggle).

```
loop {
  if paused  → await resume
  next = tasks.next_queued()          // status=Queued, ORDER BY queued_at ASC, LIMIT 1
  match next {
    None    → worker.notified().await // sleep until enqueue / resume
    Some(t) → run_pipeline(t).await   // drive ONE task fully, then loop
  }
}
```

`enqueue_task` and `resume` call `notify_one()` to wake a sleeping worker.

**Pause semantics:** pausing stops pulling **new** tasks; an in-flight task **runs to its terminal state** (no mid-run kill). Commands: `pause_dispatch` / `resume_dispatch` / `get_dispatch_status`.

### `ProcessRunner` change (and the reaper-fix bonus)

Today `spawn`'s reaper only *emits* `task-exit` to the frontend; Rust cannot await completion. Add a per-task completion channel: the reaper (which already `wait()`s on the child) resolves a stored `oneshot`/`watch` so callers can do:

```
runner.wait_for_exit(task_id).await -> ExitInfo   // { exit_code: Option<i32>, signaled: bool, stopped_by_user: bool }
```

The worker awaits this instead of depending on the frontend `task-exit` listener — the Rust-side reconcile the earlier fragility note called for. The existing `task-exit` event **still fires** (live terminal + manual-run reconcile untouched). `stopped_by_user` is set when `ProcessRunner::stop` was invoked, so the worker can distinguish a human Stop from a crash.

### Pipeline internals — `run_pipeline(task)`

1. **Worktree** — `create_worktree_lifecycle(...)` → `WorktreeCreated`.
2. **Agent** — `ClaudeCodeAdapter::start(...)` → `Running`; then `runner.wait_for_exit(task_id).await`. Outcome = **error** if spawn failed or exit was nonzero/signaled (and not a user Stop); **success** if exit `0`.
3. **Reconcile** — `git::status` on the worktree: no changes → `Stopped`; changes → continue.
4. **Verification** — `VerificationService::run_for_task` → `VerificationRunning`, persists a `VerificationRun`. Gate: **any check `Failed` blocks the PR** (warnings/skipped are fine).
5. **Draft PR** — reuse `pr::create_pr` with `draft: true` (the wire field already exists) → push branch + `gh pr create --draft` → `PrPrepared`.

### Retry / failure matrix

| What happened | Retry? | Lands in | PR? |
|---|---|---|---|
| Worktree/setup error (deterministic, e.g. path exists, not a git repo) | no | `Failed` (+ error) | — |
| Agent run errored | retry agent once in same worktree → still errors | `Failed` | — |
| Agent ran, no file changes | no | `Stopped` | — |
| Verification has a `Failed` check | re-run verification once (flaky guard) → still red | `ReviewReady` | no |
| Verification clean, push/`gh` step errored | retry once → still fails | `ReviewReady` (+ error) | no |
| Verification clean, PR opened | — | `PrPrepared` | ✅ draft |
| Human clicks Stop on a queued run | no | `Stopped` | — |

"Retry once" = **agent errors** re-run the agent; **verification red** re-runs verification only (flake guard, not a fix-loop).

**Codex guard** — enforced at `enqueue` (rejected up front) and defensively in the worker: a non-`claude-code` task that somehow reaches `Queued` is skipped → `Failed` with a clear reason, never crashing the loop.

**Visibility** — the worker emits a lightweight `dispatch-event` Tauri event (`task_id`, stage, outcome) so the UI can narrate live. Durable record = the persisted task status, `VerificationRun`, and PR URL.

## Settings

Reuse the `app_settings` table. One key under `scope='global'`: `dispatch.autorun` (bool), default **on** (safe — nothing runs until a task is enqueued). The toggle flips it; persisted so the run/pause state survives restart. Concurrency is a hard-coded 1 (no setting yet).

## UI surface (minimal)

- **Task Board card** — a **"Queue"** action (on `Draft`/`Stopped`/`Failed`/`ChangesRequested`) → `enqueue_task`; a new **`queued`** status badge; a small **"Queued (N)"** count.
- **Dispatch control** in the Task Board header — **Auto-run toggle** + live status ("Worker: running · 2 queued · #041 in progress"), fed by `get_dispatch_status` + the `dispatch-event` stream.
- **No new terminal** — a queued task runs through the same `ProcessRunner`, so selecting it shows live output in the existing Agent Workspace exactly as a manual run does.
- **Hooks:** `useEnqueueTask`, `useDequeueTask`, `useDispatchStatus`, `usePauseDispatch`/`useResumeDispatch`; bindings regenerate from Rust.

## Crash recovery

Agents are `kill_on_drop`, so quitting kills any in-flight agent. On `AppState::init`, a **startup sweep** reconciles orphans: tasks left in `Running`/`VerificationRunning` are resolved by `git::status` → `ReviewReady` (changes) or `Stopped` (clean); `Queued` tasks stay queued and the worker resumes draining (if autorun on). Interrupted tasks are **not auto-retried** — left resting for a human or a manual re-queue, so a crash never silently re-runs an agent.

## Testing

Two **seams** keep `run_pipeline` testable without network:
1. The engine is already `ProcessRunner` — tests spawn a fake short-lived script that does/doesn't write a file and exits 0/nonzero.
2. The PR step goes behind a small **`PrPublisher` trait** — real impl shells `gh`; test impl records calls + returns a fake URL.

**Rust tests:**
- `next_queued` FIFO ordering by `queued_at`.
- `enqueue`/`dequeue` status guards + codex rejection.
- `wait_for_exit` resolves correct `ExitInfo` (normal exit vs. user Stop/signaled).
- Pipeline outcomes → `PrPrepared` / `Stopped` / `ReviewReady` / `Failed`, driving verification pass/fail via configured `sh -c` commands.
- Retry rules: agent fails twice → `Failed`; verification red twice → `ReviewReady`.
- Pause blocks new pulls.
- Startup sweep reconciles a seeded `Running` task (dirty → `ReviewReady`, clean → `Stopped`).

**Frontend (Vitest):** Queue button, Auto-run toggle, dispatch status display, with the new commands mocked in `lib/tauri.ts` like the rest.

## Files touched (anticipated)

- `src-tauri/migrations/20260104000000_dispatch_queue.sql` — `queued_at` column.
- `src-tauri/src/models.rs` — `TaskStatus::Queued`; `Task.queued_at`.
- `src-tauri/src/process/mod.rs` — `wait_for_exit` + `ExitInfo` (with `stopped_by_user`).
- `src-tauri/src/dispatch/` (new) — `DispatchWorker`, `run_pipeline`, `PrPublisher` trait, retry logic, startup sweep.
- `src-tauri/src/commands/dispatch.rs` (new) — `enqueue_task`, `dequeue_task`, `get_dispatch_status`, `pause_dispatch`, `resume_dispatch`.
- `src-tauri/src/tasks/` — `next_queued`, enqueue/dequeue persistence.
- `src-tauri/src/state.rs`, `src-tauri/src/lib.rs` — wire the worker + register commands/events + startup sweep.
- `features/dispatch/*` (new) — hooks.
- `components/panels/task-board/*` — Queue action, queued badge, dispatch control.
- `lib/tauri.ts` — mocks for the new commands.
