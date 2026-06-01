# Auto-Dispatch UI — Design

**Status:** Approved (2026-06-01)
**Sibling of:** [Auto-dispatch queue backend](2026-05-31-auto-dispatch-queue-design.md) (merged in PR #34)

## Goal

Surface the merged auto-dispatch backend in the frontend so a human can enqueue tasks, watch the queue drain, and pause/resume the worker — without leaving the Task Board.

## Backend contract (already shipped)

The backend is merged and unchanged by this work. The UI consumes:

- **Commands:** `enqueue_task(taskId) -> Task`, `dequeue_task(taskId) -> Task`, `get_dispatch_status() -> DispatchStatus`, `pause_dispatch() -> null`, `resume_dispatch() -> null`.
- **Event:** `dispatchEvent` with payload `{ taskId: string, stage: string, outcome: string }` (exposed as `events.dispatchEvent` in `lib/bindings.ts`).
- **Type:** `DispatchStatus = { running: boolean, queued: number, currentTask: string | null }`.
- **Task model:** `TaskStatus` now includes `"queued"`; `Task.queuedAt: string | null`.
- **Enqueue eligibility** (mirrors the pure `enqueue_eligibility` guard in `src-tauri/src/commands/dispatch.rs`): status ∈ {`draft`, `stopped`, `failed`, `changesRequested`} **and** `selectedEngine` is `"claude-code"` or `null` (null defaults to claude-code). `codex-cli` is rejected. Any other status is rejected.
- **Dequeue eligibility:** task status must be exactly `"queued"`.

The backend command stubs already exist in `lib/tauri.ts` (added in PR #34) but are static; this work replaces them with reactive mocks (see §5).

## Scope

In scope: dispatch feature hooks, a Task Board dispatch control (Auto-run toggle + worker status line), a per-task Queue action in the agent workspace, a queued status badge with a dequeue affordance, reactive dev-mode mocks, and tests.

Out of scope: changing any backend behavior; simulating the worker draining the queue in dev mode (events/status advance only in the real desktop build); multi-engine dispatch (claude-code only, enforced by the backend).

## Design decisions (locked during brainstorming)

1. **Dispatch control lives in the Task Board panel header** (PanelFrame `actions` row, beside the + button) — queueing concerns co-located with the task list.
2. **The per-task Queue action lives in the agent workspace only** (the `StartButton` flow), keeping the TaskCard a selector. The card surfaces queue *state* (badge) but not the enqueue action.
3. **Dev-mode mocks are "reactive counts only"** — an in-memory queue makes counts and badges update on enqueue/dequeue, but there is no simulated draining and no `dispatchEvent` emission.
4. **Dequeue is a click on the queued badge** (an × affordance on the badge), not a separate button.

## File structure

New files:

- `features/dispatch/use-dispatch-status.ts` — `useQuery(["dispatch-status"])` wrapping `tauri.getDispatchStatus`.
- `features/dispatch/use-dispatch-events.ts` — subscribes to `events.dispatchEvent`, invalidates `["dispatch-status"]` and `["tasks"]`.
- `features/dispatch/use-enqueue-task.ts` — `useMutation` wrapping `tauri.enqueueTask`.
- `features/dispatch/use-dequeue-task.ts` — `useMutation` wrapping `tauri.dequeueTask`.
- `features/dispatch/use-toggle-autorun.ts` — exports `usePauseDispatch` and `useResumeDispatch` mutations.
- `components/ui/switch.tsx` — shadcn Switch primitive (not yet in the repo), used for the Auto-run toggle.
- `components/panels/task-board/dispatch-control.tsx` — the Auto-run toggle + worker status line.

Modified files:

- `components/panels/task-board/index.tsx` — render `<DispatchControl />` in the PanelFrame `actions` slot beside the + button.
- `components/panels/task-board/task-card.tsx` — add `queued` to the status-color map; convert the card root to a `div role="button"`; render a dequeue × on the badge when `status === "queued"`.
- `components/panels/agent-workspace/start-button.tsx` — add the Queue button branch.
- `lib/tauri.ts` — replace the static dispatch stubs with reactive mocks backed by an in-memory queue.

## Component & data-flow design

### Hooks

All hooks follow the existing `useStartTask` pattern (see `features/runs/use-start-task.ts`): a `useMutation` whose `mutationFn` calls `tauri.X`, throws `new Error(result.error.message)` on `result.status === "error"`, and on success invalidates the relevant queries.

- **`useDispatchStatus`** — `useQuery({ queryKey: ["dispatch-status"], queryFn })`. The `queryFn` calls `tauri.getDispatchStatus()` and returns `result.data` (throwing on error). Returns the standard TanStack Query object; consumers read `data` (a `DispatchStatus`).
- **`useEnqueueTask`** — mutation over `tauri.enqueueTask(taskId)`. On success invalidates `["tasks", task.projectId]`, `["task", task.id]`, and `["dispatch-status"]`.
- **`useDequeueTask`** — mutation over `tauri.dequeueTask(taskId)`. Same invalidations as enqueue.
- **`usePauseDispatch` / `useResumeDispatch`** (in `use-toggle-autorun.ts`) — mutations over `tauri.pauseDispatch()` / `tauri.resumeDispatch()`. On success invalidate `["dispatch-status"]`.
- **`useDispatchEvents`** — mirrors `features/runs/use-task-output.ts`: a `useEffect` that `await events.dispatchEvent.listen(...)`, and on every event invalidates `["dispatch-status"]` and `["tasks"]` (so badges and the status line track the worker live). The listen is wrapped in try/catch so it silently no-ops in browser/dev mode (no Tauri runtime). Cleanup calls the returned `UnlistenFn`. It takes the `QueryClient` from `useQueryClient()` and returns nothing.

### DispatchControl

Rendered in the Task Board PanelFrame `actions` row, to the left of the existing + button. Responsibilities:

- Mounts `useDispatchEvents()` once (single consumer).
- Reads `useDispatchStatus()`. While `data` is undefined (loading), renders nothing (or a muted dash) — no spinner needed for a one-line status.
- Renders a `Switch` bound to `status.running`. Toggling on calls `useResumeDispatch().mutate()`; toggling off calls `usePauseDispatch().mutate()`. The switch is `disabled` while either mutation `isPending`.
- Renders a compact status line next to the switch:
  - `status.running === false` → `Paused`.
  - `running && queued === 0 && currentTask === null` → `Idle`.
  - otherwise → `N queued` and, when `currentTask` is non-null, ` · #<id>` where `<id>` strips the `task-` prefix (matching the TaskCard convention).
- Layout is compact (text-[11px], matching panel chrome). The switch has an accessible label "Auto-run".

### StartButton — Queue branch

`StartButton` (in `agent-workspace/start-button.tsx`) currently switches on `status` to show one primary action. Extend it:

- Compute `engineEligible = task.selectedEngine === null || task.selectedEngine === "claude-code"` and `statusEligible = status ∈ {draft, stopped, failed, changesRequested}`. (The component currently receives only `taskId` and `status`; it must also receive `selectedEngine: string | null` — update the prop type and the one call site in `agent-workspace/index.tsx`.)
- When `statusEligible`, render a secondary **Queue** button beside the existing primary button:
  - `draft` → `[Create worktree] [Queue]`
  - `stopped` / `failed` → `[Start] [Queue]`
  - `changesRequested` → `[Queue]` (this status currently renders nothing)
- The Queue button calls `useEnqueueTask().mutate(taskId)`, shows a spinner while `isPending`, and uses a `ListPlus` (or similar) lucide icon with a `variant="outline"` / `size="sm"` styling consistent with the existing buttons.
- When `!engineEligible`, the Queue button is `disabled` and wrapped in a tooltip reading "Only claude-code is dispatchable". (Use the existing `components/ui/tooltip.tsx`.)
- When `status === "queued"`, render a muted, non-interactive "Queued" indicator (e.g., a `Badge variant="outline"` with a small clock/queue icon) in place of action buttons — dequeue happens on the card, not here.

### TaskCard — queued badge + dequeue

In `task-card.tsx`:

- Add `queued: "bg-info/20 text-info"` to the `statusColor` map. (If no `info` token exists in the theme, use `bg-primary/15 text-primary` — verify against `app/globals.css` / Tailwind config during implementation and pick an existing token; do not introduce a new color token.)
- **Restructure the card root** from `<button>` to `<div role="button" tabIndex={0}>` with `onClick={onSelect}` and an `onKeyDown` handler that calls `onSelect()` on `Enter` or `Space` (and `preventDefault` on Space to avoid scroll). Keep all existing classes and the `active` styling. This is required so the dequeue control can be a valid nested `<button>` (a `<button>` inside a `<button>` is invalid HTML).
- When `task.status === "queued"`, render the status badge with a trailing dequeue **×**: a small `<button>` (lucide `X`, `h-3 w-3`) inside/adjacent to the badge whose `onClick` calls `e.stopPropagation()` then `useDequeueTask().mutate(task.id)`, with `aria-label="Remove from queue"`. The × shows a spinner or is `disabled` while the mutation `isPending`. For all other statuses the badge renders exactly as today.

## Mocks (dev mode)

`lib/tauri.ts` currently returns static dispatch data. Replace with a module-level reactive store:

```ts
const dispatchState = { paused: false, queued: new Set<string>() };
```

- **`enqueueTask(taskId)`** — if the task exists, `dispatchState.queued.add(taskId)` and return the task with `status: "queued"`, `queuedAt: new Date().toISOString()`. (Keep the existing not-found error path.)
- **`dequeueTask(taskId)`** — `dispatchState.queued.delete(taskId)` and return the task via `applyDispatchOverlay` (now a no-op for it, so it reverts to its original static status) with `queuedAt: null`. (Keep not-found error.) Returning the overlay-resolved task keeps the mutation result consistent with what `listTasks`/`getTask` will report after invalidation, rather than hard-coding `draft`.
- **`getDispatchStatus()`** — return `{ running: !dispatchState.paused, queued: dispatchState.queued.size, currentTask: null }`.
- **`pauseDispatch()` / `resumeDispatch()`** — set `dispatchState.paused = true/false`, return `{ status: "ok", data: null }`.
- **Overlay for `listTasks` / `getTask`** — when returning mock tasks, map each task: if `dispatchState.queued.has(task.id)`, override `status: "queued"` and `queuedAt` so the queued badge appears reactively after `["tasks"]` is invalidated. A small helper `applyDispatchOverlay(task)` keeps this DRY across `listTasks` and `getTask`.

This gives reactive counts and badges in browser mode (enqueue → badge appears, count increments; dequeue → reverts) without simulating draining or emitting `dispatchEvent`.

## Error handling

- All mutations surface backend errors by throwing in the `mutationFn` (existing pattern). The backend rejects ineligible enqueues/dequeues; the UI also pre-disables the Queue button for ineligible engine to avoid a predictable round-trip, but relies on the backend as the source of truth for status eligibility (the button is only shown for eligible statuses).
- `useDispatchEvents` and the events listener tolerate the absence of a Tauri runtime (try/catch, as in `use-task-output.ts`).
- `useDispatchStatus` loading/error states render as a muted placeholder in the control, never blocking the panel.

## Testing

Vitest, matching the existing co-located `*.test.tsx` convention (e.g., `changed-files-panel.test.tsx`, `run-verification-button.test.tsx`). Tests run against the real `mockImpl` from `lib/tauri.ts` (dev-mode), wrapped in a `QueryClientProvider`.

1. **`dispatch-control.test.tsx`**
   - Renders `Paused` when status `running` is false.
   - Renders `Idle` when running with `queued === 0` and no current task.
   - Renders `N queued · #id` when queued and a current task is set.
   - Toggling the switch calls pause when on→off and resume when off→on (assert via the mock's effect on a subsequent `getDispatchStatus`, or spy).
2. **`start-button.test.tsx`** (new)
   - Shows a Queue button for a `draft` claude-code task.
   - Queue button is disabled for a `codex-cli` task (and tooltip label present).
   - Shows "Queued" indicator (no action buttons) for a `queued` task.
3. **`task-card.test.tsx`** (new)
   - Renders the queued badge with a dequeue × when `status === "queued"`.
   - Clicking the × fires dequeue and does **not** trigger card `onSelect` (stopPropagation).
   - Renders a plain badge (no ×) for non-queued statuses.

Each task in the implementation plan follows TDD: write the failing test, watch it fail, implement, watch it pass, commit. After implementation, the full suite (`pnpm test`), `pnpm typecheck`, and `pnpm lint` must be green, and `pnpm gen:bindings` must report no diff (this work adds no Rust, so bindings are unchanged).

## Non-goals / follow-ups

- Simulated queue draining in dev mode (events + status advancing over timers) — deferred; "reactive counts only" was chosen.
- Surfacing per-task dispatch history / the `dispatchEvent` stage stream in the UI — not in this pass.
- The backend follow-ups flagged in PR #34 (dequeue TOCTOU, stop_task status guard, redundant `git status` in publish, `worker_tests` temp-dir leak) are independent and not part of this UI work.
