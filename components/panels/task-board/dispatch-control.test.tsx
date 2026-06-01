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
