# Auto-Dispatch UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface the merged auto-dispatch backend in the frontend — enqueue tasks, watch the queue drain, pause/resume the worker — without leaving the Task Board.

**Architecture:** Five thin TanStack-Query hooks in `features/dispatch/` wrap the already-shipped Tauri commands. A `DispatchControl` (Auto-run `Switch` + worker status line) mounts in the Task Board panel header. The agent-workspace `StartButton` gains a Queue action; the `TaskCard` gains a `queued` badge with a dequeue ×. Dev-mode mocks in `lib/tauri.ts` become reactive (in-memory queue) so counts/badges update in the browser. No Rust changes.

**Tech Stack:** Next.js 16 / React 19, TanStack Query, Zustand, shadcn/ui (radix-ui unified package), Tailwind, lucide-react, Vitest + Testing Library, Tauri 2 + tauri-specta bindings.

**Spec:** `docs/superpowers/specs/2026-06-01-auto-dispatch-ui-design.md`

---

## Conventions (read before starting)

- **Hooks** follow `features/runs/use-start-task.ts`: a `useMutation`/`useQuery` whose fn calls `tauri.X`, throws `new Error(result.error.message)` on `result.status === "error"`, returns `result.data`.
- **Tests** follow `components/panels/review-room/run-verification-button.test.tsx`: mock the feature hooks with `vi.mock`, render the component in isolation, assert with `@testing-library/react` + `@testing-library/jest-dom/vitest`. There is **no** `QueryClientProvider` test harness in this repo — do not introduce one. Hooks themselves are not unit-tested (verified via typecheck + downstream component tests), matching the existing `features/` directory which has no hook tests.
- **Run a single test file:** `pnpm vitest run <path>`. **Full suite:** `pnpm test`. **Types:** `pnpm typecheck`. **Lint:** `pnpm lint`.
- **No theme `info` token exists** — `app/globals.css` defines `success`, `warning`, `destructive`, plus `primary`/`muted`/`accent`. The queued badge uses `bg-primary/15 text-primary`.
- **`radix-ui` is the unified package** (`"radix-ui": "^1.4.3"`). Import primitives as `import { Switch as SwitchPrimitive } from "radix-ui"` (see `components/ui/checkbox.tsx`).
- **Bindings are unchanged** by this work (no Rust). `pnpm gen:bindings` should produce no diff.

---

## File Structure

Create:
- `lib/tauri.ts` changes (reactive mocks) — Task 1
- `features/dispatch/use-dispatch-status.ts` — Task 2
- `features/dispatch/use-enqueue-task.ts` — Task 2
- `features/dispatch/use-dequeue-task.ts` — Task 2
- `features/dispatch/use-toggle-autorun.ts` — Task 2
- `features/dispatch/use-dispatch-events.ts` — Task 2
- `components/ui/switch.tsx` — Task 3
- `components/panels/task-board/dispatch-control.tsx` (+ `.test.tsx`) — Task 4
- `components/panels/agent-workspace/start-button.test.tsx` — Task 5
- `components/panels/task-board/task-card.test.tsx` — Task 6

Modify:
- `components/panels/agent-workspace/start-button.tsx` + its call site `components/panels/agent-workspace/index.tsx:34` — Task 5
- `components/panels/task-board/task-card.tsx` — Task 6
- `components/panels/task-board/index.tsx` — Task 7

---

## Task 1: Reactive dev-mode mocks in `lib/tauri.ts`

Replace the static dispatch stubs with an in-memory queue so browser mode reflects enqueue/dequeue. This is the foundation the components demo against.

**Files:**
- Modify: `lib/tauri.ts`
- Test: `lib/tauri.test.ts`

- [ ] **Step 1: Write the failing tests**

Append to `lib/tauri.test.ts` (inside the existing `describe` block, before its closing `});`):

```ts
  it("enqueue then dequeue updates status count and task overlay reactively", async () => {
    const before = await tauri.getDispatchStatus();
    if (before.status !== "ok") throw new Error("status not ok");
    const baseCount = before.data.queued;

    const enq = await tauri.enqueueTask("task-040");
    if (enq.status !== "ok") throw new Error("enqueue not ok");
    expect(enq.data.status).toBe("queued");
    expect(enq.data.queuedAt).not.toBeNull();

    const mid = await tauri.getDispatchStatus();
    if (mid.status !== "ok") throw new Error("status not ok");
    expect(mid.data.queued).toBe(baseCount + 1);

    const listed = await tauri.listTasks("proj-default");
    if (listed.status !== "ok") throw new Error("list not ok");
    expect(listed.data.find((t) => t.id === "task-040")?.status).toBe("queued");

    // Dequeue cleans up module-level state so other tests are unaffected.
    const deq = await tauri.dequeueTask("task-040");
    if (deq.status !== "ok") throw new Error("dequeue not ok");
    expect(deq.data.status).not.toBe("queued");
    expect(deq.data.queuedAt).toBeNull();

    const after = await tauri.getDispatchStatus();
    if (after.status !== "ok") throw new Error("status not ok");
    expect(after.data.queued).toBe(baseCount);
  });

  it("pause and resume flip running in dispatch status", async () => {
    await tauri.pauseDispatch();
    const paused = await tauri.getDispatchStatus();
    if (paused.status !== "ok") throw new Error("status not ok");
    expect(paused.data.running).toBe(false);

    await tauri.resumeDispatch();
    const resumed = await tauri.getDispatchStatus();
    if (resumed.status !== "ok") throw new Error("status not ok");
    expect(resumed.data.running).toBe(true);
  });
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `pnpm vitest run lib/tauri.test.ts`
Expected: FAIL — the new "enqueue then dequeue" test fails because the static `getDispatchStatus` always returns `queued: 0` (count never increments) and `listTasks` never reports `queued`.

- [ ] **Step 3: Add the reactive store and overlay helper**

In `lib/tauri.ts`, change the type import on line 10 to include `Task`:

```ts
import type { AppError, Result, Task } from "./bindings";
```

Immediately above `const mockImpl: Commands = {` (currently line 14), add:

```ts
// Dev-mode dispatch state: reactive counts only (no simulated draining / events).
const dispatchState = { paused: false, queued: new Set<string>() };

function applyDispatchOverlay(task: Task): Task {
  if (dispatchState.queued.has(task.id)) {
    return { ...task, status: "queued", queuedAt: task.queuedAt ?? new Date().toISOString() };
  }
  return task;
}
```

- [ ] **Step 4: Apply the overlay in `listTasks` and `getTask`**

Replace the `listTasks` body (lines 19-22) with:

```ts
  listTasks: async (projectId: string): Promise<Result<Task[], AppError>> => {
    await sleep(50);
    return {
      status: "ok",
      data: mockTasks.filter((t) => t.projectId === projectId).map(applyDispatchOverlay),
    };
  },
```

In `getTask`, replace the success return (currently line 36, `return { status: "ok", data: task };`) with:

```ts
    return { status: "ok", data: applyDispatchOverlay(task) };
```

- [ ] **Step 5: Make the dispatch commands reactive**

Replace the five dispatch stubs (currently `enqueueTask` through `resumeDispatch`, lines 224-251) with:

```ts
  enqueueTask: async (taskId: string) => {
    await sleep(60);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return { status: "error" as const, error: { code: "notFound" as const, message: `task ${taskId} not found`, details: null } };
    }
    dispatchState.queued.add(taskId);
    return { status: "ok" as const, data: applyDispatchOverlay({ ...task, queuedAt: new Date().toISOString() }) };
  },
  dequeueTask: async (taskId: string) => {
    await sleep(60);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return { status: "error" as const, error: { code: "notFound" as const, message: `task ${taskId} not found`, details: null } };
    }
    dispatchState.queued.delete(taskId);
    // Overlay is now a no-op for this id, so it reverts to its original static status.
    return { status: "ok" as const, data: applyDispatchOverlay({ ...task, queuedAt: null }) };
  },
  getDispatchStatus: async () => {
    await sleep(30);
    return {
      status: "ok" as const,
      data: { running: !dispatchState.paused, queued: dispatchState.queued.size, currentTask: null },
    };
  },
  pauseDispatch: async () => {
    await sleep(20);
    dispatchState.paused = true;
    return { status: "ok" as const, data: null };
  },
  resumeDispatch: async () => {
    await sleep(20);
    dispatchState.paused = false;
    return { status: "ok" as const, data: null };
  },
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `pnpm vitest run lib/tauri.test.ts`
Expected: PASS (all tests, including the two new ones).

- [ ] **Step 7: Typecheck**

Run: `pnpm typecheck`
Expected: no errors.

- [ ] **Step 8: Commit**

```bash
git add lib/tauri.ts lib/tauri.test.ts
git commit -m "feat(dispatch-ui): reactive dev-mode mocks for the dispatch queue"
```

---

## Task 2: Dispatch feature hooks

Five thin hooks wrapping the shipped commands + the event stream. No standalone tests (repo convention — verified by typecheck here and by component tests in Tasks 4-6).

**Files:**
- Create: `features/dispatch/use-dispatch-status.ts`
- Create: `features/dispatch/use-enqueue-task.ts`
- Create: `features/dispatch/use-dequeue-task.ts`
- Create: `features/dispatch/use-toggle-autorun.ts`
- Create: `features/dispatch/use-dispatch-events.ts`

- [ ] **Step 1: Create `use-dispatch-status.ts`**

```ts
import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useDispatchStatus() {
  return useQuery({
    queryKey: ["dispatch-status"],
    queryFn: async () => {
      const result = await tauri.getDispatchStatus();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  });
}
```

- [ ] **Step 2: Create `use-enqueue-task.ts`**

```ts
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useEnqueueTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (taskId: string) => {
      const result = await tauri.enqueueTask(taskId);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (task) => {
      queryClient.invalidateQueries({ queryKey: ["tasks", task.projectId] });
      queryClient.invalidateQueries({ queryKey: ["task", task.id] });
      queryClient.invalidateQueries({ queryKey: ["dispatch-status"] });
    },
  });
}
```

- [ ] **Step 3: Create `use-dequeue-task.ts`**

```ts
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useDequeueTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (taskId: string) => {
      const result = await tauri.dequeueTask(taskId);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (task) => {
      queryClient.invalidateQueries({ queryKey: ["tasks", task.projectId] });
      queryClient.invalidateQueries({ queryKey: ["task", task.id] });
      queryClient.invalidateQueries({ queryKey: ["dispatch-status"] });
    },
  });
}
```

- [ ] **Step 4: Create `use-toggle-autorun.ts`**

```ts
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function usePauseDispatch() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      const result = await tauri.pauseDispatch();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["dispatch-status"] }),
  });
}

export function useResumeDispatch() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      const result = await tauri.resumeDispatch();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["dispatch-status"] }),
  });
}
```

- [ ] **Step 5: Create `use-dispatch-events.ts`**

Mirrors `features/runs/use-task-output.ts` — subscribe to the event stream, invalidate on each event, tolerate a missing Tauri runtime. Invalidating `["tasks"]` (no projectId) partial-matches every `["tasks", projectId]` query.

```ts
import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { events } from "@/lib/bindings";

export function useDispatchEvents() {
  const queryClient = useQueryClient();
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      try {
        unlisten = await events.dispatchEvent.listen(() => {
          queryClient.invalidateQueries({ queryKey: ["dispatch-status"] });
          queryClient.invalidateQueries({ queryKey: ["tasks"] });
        });
      } catch {
        // Dev mode without Tauri runtime — no event stream, fine.
      }
    })();
    return () => unlisten?.();
  }, [queryClient]);
}
```

- [ ] **Step 6: Typecheck**

Run: `pnpm typecheck`
Expected: no errors.

- [ ] **Step 7: Commit**

```bash
git add features/dispatch/
git commit -m "feat(dispatch-ui): feature hooks for status, enqueue, dequeue, autorun, events"
```

---

## Task 3: `Switch` UI primitive

The Auto-run toggle needs a `Switch`; the repo doesn't have one yet. Add the canonical shadcn Switch built on the unified `radix-ui` package. No test (vendored primitive, matching `components/ui/checkbox.tsx` which has none).

**Files:**
- Create: `components/ui/switch.tsx`

- [ ] **Step 1: Create `components/ui/switch.tsx`**

```tsx
import * as React from "react"
import { Switch as SwitchPrimitive } from "radix-ui"

import { cn } from "@/lib/utils"

function Switch({
  className,
  ...props
}: React.ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      className={cn(
        "peer data-[state=checked]:bg-primary data-[state=unchecked]:bg-input focus-visible:border-ring focus-visible:ring-ring/50 dark:data-[state=unchecked]:bg-input/80 inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs transition-all outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          "bg-background dark:data-[state=unchecked]:bg-foreground dark:data-[state=checked]:bg-primary-foreground pointer-events-none block size-4 rounded-full ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0"
        )}
      />
    </SwitchPrimitive.Root>
  )
}

export { Switch }
```

- [ ] **Step 2: Typecheck**

Run: `pnpm typecheck`
Expected: no errors. (If `radix-ui` doesn't export `Switch`, confirm the package version with `grep radix-ui package.json` — `^1.4.3` exports it.)

- [ ] **Step 3: Commit**

```bash
git add components/ui/switch.tsx
git commit -m "feat(ui): add shadcn Switch primitive"
```

---

## Task 4: `DispatchControl` component

The Auto-run toggle + worker status line, mounted later (Task 7) in the Task Board header.

**Files:**
- Create: `components/panels/task-board/dispatch-control.tsx`
- Test: `components/panels/task-board/dispatch-control.test.tsx`

- [ ] **Step 1: Write the failing test**

`components/panels/task-board/dispatch-control.test.tsx`:

```tsx
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

const pauseMock = vi.fn();
const resumeMock = vi.fn();
const useDispatchStatusMock = vi.fn();

vi.mock("@/features/dispatch/use-dispatch-events", () => ({
  useDispatchEvents: () => {},
}));
vi.mock("@/features/dispatch/use-dispatch-status", () => ({
  useDispatchStatus: () => useDispatchStatusMock(),
}));
vi.mock("@/features/dispatch/use-toggle-autorun", () => ({
  usePauseDispatch: () => ({ mutate: pauseMock, isPending: false }),
  useResumeDispatch: () => ({ mutate: resumeMock, isPending: false }),
}));

import { DispatchControl } from "./dispatch-control";

afterEach(() => {
  cleanup();
  pauseMock.mockReset();
  resumeMock.mockReset();
  useDispatchStatusMock.mockReset();
});

describe("DispatchControl", () => {
  it("shows Paused when the worker is not running", () => {
    useDispatchStatusMock.mockReturnValue({ data: { running: false, queued: 2, currentTask: null } });
    render(<DispatchControl />);
    expect(screen.getByText("Paused")).toBeInTheDocument();
  });

  it("shows Idle when running with an empty queue and no current task", () => {
    useDispatchStatusMock.mockReturnValue({ data: { running: true, queued: 0, currentTask: null } });
    render(<DispatchControl />);
    expect(screen.getByText("Idle")).toBeInTheDocument();
  });

  it("shows the queued count and current task id", () => {
    useDispatchStatusMock.mockReturnValue({ data: { running: true, queued: 3, currentTask: "task-042" } });
    render(<DispatchControl />);
    expect(screen.getByText("3 queued · #042")).toBeInTheDocument();
  });

  it("pauses when toggled off and resumes when toggled on", () => {
    useDispatchStatusMock.mockReturnValue({ data: { running: true, queued: 0, currentTask: null } });
    const { rerender } = render(<DispatchControl />);
    fireEvent.click(screen.getByRole("switch"));
    expect(pauseMock).toHaveBeenCalled();

    useDispatchStatusMock.mockReturnValue({ data: { running: false, queued: 0, currentTask: null } });
    rerender(<DispatchControl />);
    fireEvent.click(screen.getByRole("switch"));
    expect(resumeMock).toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm vitest run components/panels/task-board/dispatch-control.test.tsx`
Expected: FAIL — `Cannot find module './dispatch-control'`.

- [ ] **Step 3: Implement `dispatch-control.tsx`**

```tsx
"use client";

import { Switch } from "@/components/ui/switch";
import { useDispatchStatus } from "@/features/dispatch/use-dispatch-status";
import { usePauseDispatch, useResumeDispatch } from "@/features/dispatch/use-toggle-autorun";
import { useDispatchEvents } from "@/features/dispatch/use-dispatch-events";

export function DispatchControl() {
  useDispatchEvents();
  const { data: status } = useDispatchStatus();
  const pause = usePauseDispatch();
  const resume = useResumeDispatch();

  const running = status?.running ?? false;
  const pending = pause.isPending || resume.isPending;

  function onToggle(next: boolean) {
    if (next) resume.mutate();
    else pause.mutate();
  }

  let line: string;
  if (!status) {
    line = "…";
  } else if (!status.running) {
    line = "Paused";
  } else if (status.queued === 0 && status.currentTask === null) {
    line = "Idle";
  } else {
    line = `${status.queued} queued`;
    if (status.currentTask) line += ` · #${status.currentTask.replace("task-", "")}`;
  }

  return (
    <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
      <Switch
        aria-label="Auto-run"
        checked={running}
        onCheckedChange={onToggle}
        disabled={pending || !status}
      />
      <span className="tabular-nums">{line}</span>
    </div>
  );
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm vitest run components/panels/task-board/dispatch-control.test.tsx`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add components/panels/task-board/dispatch-control.tsx components/panels/task-board/dispatch-control.test.tsx
git commit -m "feat(dispatch-ui): DispatchControl auto-run toggle + worker status line"
```

---

## Task 5: `StartButton` Queue action

Add a Queue button to the agent-workspace action surface and a "Queued" indicator, gated on the backend's enqueue eligibility (status + engine).

**Files:**
- Modify: `components/panels/agent-workspace/start-button.tsx`
- Modify: `components/panels/agent-workspace/index.tsx:34`
- Test: `components/panels/agent-workspace/start-button.test.tsx`

- [ ] **Step 1: Write the failing test**

`components/panels/agent-workspace/start-button.test.tsx`:

```tsx
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

const enqueueMock = vi.fn();

vi.mock("@/features/worktrees/use-create-worktree", () => ({
  useCreateWorktree: () => ({ mutate: vi.fn(), isPending: false }),
}));
vi.mock("@/features/runs/use-start-task", () => ({
  useStartTask: () => ({ mutate: vi.fn(), isPending: false }),
}));
vi.mock("@/features/runs/use-stop-task", () => ({
  useStopTask: () => ({ mutate: vi.fn(), isPending: false }),
}));
vi.mock("@/features/dispatch/use-enqueue-task", () => ({
  useEnqueueTask: () => ({ mutate: enqueueMock, isPending: false }),
}));

import { StartButton } from "./start-button";

afterEach(() => {
  cleanup();
  enqueueMock.mockReset();
});

describe("StartButton queue action", () => {
  it("shows an enabled Queue button for a draft claude-code task", () => {
    render(<StartButton taskId="task-1" status="draft" selectedEngine="claude-code" />);
    expect(screen.getByRole("button", { name: /^Queue$/i })).toBeEnabled();
  });

  it("disables Queue for a codex-cli task and explains why", () => {
    render(<StartButton taskId="task-1" status="draft" selectedEngine="codex-cli" />);
    expect(screen.getByRole("button", { name: /^Queue$/i })).toBeDisabled();
    expect(screen.getByTitle(/only claude-code is dispatchable/i)).toBeInTheDocument();
  });

  it("shows a non-interactive Queued indicator for a queued task", () => {
    render(<StartButton taskId="task-1" status="queued" selectedEngine="claude-code" />);
    expect(screen.getByText("Queued")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Queue$/i })).not.toBeInTheDocument();
  });

  it("enqueues with the task id on click (null engine is eligible)", () => {
    render(<StartButton taskId="task-7" status="draft" selectedEngine={null} />);
    fireEvent.click(screen.getByRole("button", { name: /^Queue$/i }));
    expect(enqueueMock).toHaveBeenCalledWith("task-7");
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm vitest run components/panels/agent-workspace/start-button.test.tsx`
Expected: FAIL — the existing `StartButton` doesn't accept `selectedEngine`, renders no Queue button, and has no "Queued" branch.

- [ ] **Step 3: Rewrite `start-button.tsx`**

```tsx
"use client";

import type { ReactNode } from "react";
import { Loader2, GitBranch, Play, Square, ListPlus, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useCreateWorktree } from "@/features/worktrees/use-create-worktree";
import { useStartTask } from "@/features/runs/use-start-task";
import { useStopTask } from "@/features/runs/use-stop-task";
import { useEnqueueTask } from "@/features/dispatch/use-enqueue-task";
import type { TaskStatus } from "@/lib/bindings";

interface Props {
  taskId: string;
  status: TaskStatus;
  selectedEngine: string | null;
}

const ENQUEUE_ELIGIBLE: TaskStatus[] = ["draft", "stopped", "failed", "changesRequested"];

export function StartButton({ taskId, status, selectedEngine }: Props) {
  const createWorktree = useCreateWorktree();
  const startTask = useStartTask();
  const stopTask = useStopTask();
  const enqueueTask = useEnqueueTask();

  if (status === "queued") {
    return (
      <Badge variant="outline" className="gap-1 text-[11px] text-muted-foreground">
        <Clock className="h-3 w-3" /> Queued
      </Badge>
    );
  }

  const engineEligible = selectedEngine === null || selectedEngine === "claude-code";
  const statusEligible = ENQUEUE_ELIGIBLE.includes(status);

  const queueButton = statusEligible ? (
    <span title={engineEligible ? undefined : "Only claude-code is dispatchable"}>
      <Button
        size="sm"
        variant="outline"
        onClick={() => enqueueTask.mutate(taskId)}
        disabled={!engineEligible || enqueueTask.isPending}
      >
        {enqueueTask.isPending ? (
          <Loader2 className="mr-1 h-3 w-3 animate-spin" />
        ) : (
          <ListPlus className="mr-1 h-3 w-3" />
        )}
        Queue
      </Button>
    </span>
  ) : null;

  let primary: ReactNode = null;
  if (status === "draft") {
    primary = (
      <Button size="sm" onClick={() => createWorktree.mutate(taskId)} disabled={createWorktree.isPending}>
        {createWorktree.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <GitBranch className="mr-1 h-3 w-3" />}
        Create worktree
      </Button>
    );
  } else if (status === "worktreeCreated" || status === "stopped" || status === "failed") {
    primary = (
      <Button size="sm" onClick={() => startTask.mutate(taskId)} disabled={startTask.isPending}>
        {startTask.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <Play className="mr-1 h-3 w-3" />}
        Start
      </Button>
    );
  } else if (status === "running" || status === "verificationRunning") {
    primary = (
      <Button size="sm" variant="destructive" onClick={() => stopTask.mutate(taskId)} disabled={stopTask.isPending}>
        {stopTask.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <Square className="mr-1 h-3 w-3" />}
        Stop
      </Button>
    );
  }

  if (!primary && !queueButton) return null;

  return (
    <div className="flex items-center gap-2">
      {primary}
      {queueButton}
    </div>
  );
}
```

Note on eligibility table: `draft` → `[Create worktree] [Queue]`; `stopped`/`failed` → `[Start] [Queue]`; `changesRequested` → `[Queue]` only (no primary); `worktreeCreated` → `[Start]` only (not enqueue-eligible); `running`/`verificationRunning` → `[Stop]` only.

- [ ] **Step 4: Update the call site**

In `components/panels/agent-workspace/index.tsx`, change line 34 from:

```tsx
          <StartButton taskId={task.id} status={task.status} />
```

to:

```tsx
          <StartButton taskId={task.id} status={task.status} selectedEngine={task.selectedEngine} />
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `pnpm vitest run components/panels/agent-workspace/start-button.test.tsx`
Expected: PASS (4 tests).

- [ ] **Step 6: Typecheck**

Run: `pnpm typecheck`
Expected: no errors (confirms the call site matches the new prop type).

- [ ] **Step 7: Commit**

```bash
git add components/panels/agent-workspace/start-button.tsx components/panels/agent-workspace/start-button.test.tsx components/panels/agent-workspace/index.tsx
git commit -m "feat(dispatch-ui): Queue action + Queued indicator in StartButton"
```

---

## Task 6: `TaskCard` queued badge + dequeue

Add the `queued` badge color, convert the card root so a dequeue button can be legally nested, and wire the dequeue ×.

**Files:**
- Modify: `components/panels/task-board/task-card.tsx`
- Test: `components/panels/task-board/task-card.test.tsx`

- [ ] **Step 1: Write the failing test**

`components/panels/task-board/task-card.test.tsx`:

```tsx
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import type { Task } from "@/lib/tauri";

const dequeueMock = vi.fn();
vi.mock("@/features/dispatch/use-dequeue-task", () => ({
  useDequeueTask: () => ({ mutate: dequeueMock, isPending: false }),
}));

import { TaskCard } from "./task-card";

const baseTask: Task = {
  id: "task-040",
  projectId: "proj-default",
  title: "Reduce dashboard query latency",
  description: "",
  outOfScope: "",
  filesToTouchHint: "",
  acceptanceCriteria: [],
  constraints: [],
  selectedEngine: "claude-code",
  status: "draft",
  risk: "safe",
  branchName: null,
  worktreePath: null,
  createdAt: "2026-05-17T09:00:00Z",
  queuedAt: null,
};

afterEach(() => {
  cleanup();
  dequeueMock.mockReset();
});

describe("TaskCard queued state", () => {
  it("renders a dequeue control when the task is queued", () => {
    render(
      <TaskCard
        task={{ ...baseTask, status: "queued", queuedAt: "2026-05-17T10:00:00Z" }}
        active={false}
        onSelect={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: /remove from queue/i })).toBeInTheDocument();
  });

  it("does not render a dequeue control for a non-queued task", () => {
    render(<TaskCard task={baseTask} active={false} onSelect={() => {}} />);
    expect(screen.queryByRole("button", { name: /remove from queue/i })).not.toBeInTheDocument();
  });

  it("dequeues without selecting the card when the × is clicked", () => {
    const onSelect = vi.fn();
    render(<TaskCard task={{ ...baseTask, status: "queued" }} active={false} onSelect={onSelect} />);
    fireEvent.click(screen.getByRole("button", { name: /remove from queue/i }));
    expect(dequeueMock).toHaveBeenCalledWith("task-040");
    expect(onSelect).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm vitest run components/panels/task-board/task-card.test.tsx`
Expected: FAIL — there is no "remove from queue" control today.

- [ ] **Step 3: Rewrite `task-card.tsx`**

```tsx
"use client";

import { X, Clock } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { useDequeueTask } from "@/features/dispatch/use-dequeue-task";
import type { Task } from "@/lib/tauri";

const statusColor: Record<string, string> = {
  draft: "bg-muted text-muted-foreground",
  queued: "bg-primary/15 text-primary",
  running: "bg-warning/20 text-warning",
  reviewReady: "bg-primary/20 text-primary",
  approved: "bg-success/20 text-success",
  changesRequested: "bg-destructive/20 text-destructive",
};

interface TaskCardProps {
  task: Task;
  active: boolean;
  onSelect: () => void;
}

export function TaskCard({ task, active, onSelect }: TaskCardProps) {
  const dequeue = useDequeueTask();
  const isQueued = task.status === "queued";

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        "w-full cursor-pointer rounded-lg border border-border/60 bg-card/60 p-3 text-left transition-colors",
        "hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active && "border-primary/60 bg-primary/10",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
          #{task.id.replace("task-", "")}
        </span>
        <Badge className={cn("gap-1 text-[10px]", statusColor[task.status] ?? "bg-muted")}>
          {isQueued && <Clock className="h-2.5 w-2.5" />}
          {task.status}
          {isQueued && (
            <button
              type="button"
              aria-label="Remove from queue"
              className="ml-0.5 rounded-sm hover:bg-primary/20 disabled:opacity-50"
              disabled={dequeue.isPending}
              onClick={(e) => {
                e.stopPropagation();
                dequeue.mutate(task.id);
              }}
            >
              <X className="h-2.5 w-2.5" />
            </button>
          )}
        </Badge>
      </div>
      <p className="mt-1.5 line-clamp-2 text-sm leading-snug">{task.title}</p>
      {task.selectedEngine && (
        <p className="mt-2 text-[11px] text-muted-foreground">via {task.selectedEngine}</p>
      )}
    </div>
  );
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm vitest run components/panels/task-board/task-card.test.tsx`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add components/panels/task-board/task-card.tsx components/panels/task-board/task-card.test.tsx
git commit -m "feat(dispatch-ui): queued badge + dequeue control on TaskCard"
```

---

## Task 7: Mount `DispatchControl` in the Task Board header + final gate

Wire the control into the panel header and run the full green gate.

**Files:**
- Modify: `components/panels/task-board/index.tsx`

- [ ] **Step 1: Mount the control in the PanelFrame actions slot**

In `components/panels/task-board/index.tsx`, add the import after the `NewTaskDialog` import (line 10):

```tsx
import { DispatchControl } from "./dispatch-control";
```

Replace the `actions` prop (currently lines 24-35, the lone `<Button>`) with a flex row containing the control and the existing + button:

```tsx
        actions={
          <div className="flex items-center gap-2">
            <DispatchControl />
            <Button
              size="icon"
              variant="ghost"
              className="h-7 w-7"
              aria-label="Add task"
              onClick={() => setOpen(true)}
              disabled={!activeProjectId}
            >
              <Plus className="h-4 w-4" />
            </Button>
          </div>
        }
```

- [ ] **Step 2: Typecheck**

Run: `pnpm typecheck`
Expected: no errors.

- [ ] **Step 3: Run the full test suite**

Run: `pnpm test`
Expected: PASS — all prior tests plus the new dispatch tests (no regressions).

- [ ] **Step 4: Lint**

Run: `pnpm lint`
Expected: 0 errors.

- [ ] **Step 5: Verify bindings are unchanged**

Run: `pnpm gen:bindings && git diff --stat lib/bindings.ts`
Expected: no diff (this work adds no Rust). If `cargo` is unavailable in the environment, skip this step and note it.

- [ ] **Step 6: Commit**

```bash
git add components/panels/task-board/index.tsx
git commit -m "feat(dispatch-ui): mount DispatchControl in the Task Board header"
```

---

## Self-Review (completed during planning)

**Spec coverage:**
- Hooks (`useDispatchStatus`/`useEnqueueTask`/`useDequeueTask`/`usePause`+`useResumeDispatch`/`useDispatchEvents`) → Task 2. ✅
- Dispatch control (Auto-run toggle + status line, in Task Board header) → Tasks 3, 4, 7. ✅
- Queue action (agent-workspace only, engine + status gated) → Task 5. ✅
- Queued badge + dequeue × on card → Task 6. ✅
- Reactive dev-mode mocks → Task 1. ✅
- Tests for control / start-button / task-card → Tasks 4, 5, 6; mock reactivity → Task 1. ✅
- `Switch` primitive prerequisite → Task 3. ✅

**Type consistency:** `DispatchStatus {running, queued, currentTask}`, `Task.status === "queued"`, `Task.queuedAt`, and the `StartButton` `selectedEngine: string | null` prop are used identically across tasks and match `lib/bindings.ts`. Query keys (`["dispatch-status"]`, `["tasks", projectId]`, `["task", id]`, partial `["tasks"]`) are consistent across hooks and the event invalidator.

**Placeholder scan:** No TBD/TODO; every code step contains complete code; every command has an expected result.

**Deviations from spec (intentional, documented):**
- The disabled-Queue reason uses a native `title` attribute on a wrapping `<span>` (not `components/ui/tooltip.tsx`). Rationale: deterministic to assert in tests without radix's hover/portal async; satisfies the spec's intent of explaining why the action is disabled.
- Hooks have no standalone unit tests — the repo has no hook-test harness and tests components by mocking hooks; hook correctness is covered by typecheck + the component tests.
