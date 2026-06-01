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
