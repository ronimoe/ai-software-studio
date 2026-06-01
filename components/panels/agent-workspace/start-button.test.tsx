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

  it("renders only the Queue button (no primary action) for changesRequested", () => {
    render(<StartButton taskId="task-1" status="changesRequested" selectedEngine="claude-code" />);
    expect(screen.getByRole("button", { name: /^Queue$/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /create worktree/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Start$/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Stop$/i })).not.toBeInTheDocument();
  });
});
