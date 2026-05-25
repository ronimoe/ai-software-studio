import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

const mutateMock = vi.fn();
const useRunVerificationMock = vi.fn();

vi.mock("@/features/verification/use-run-verification", () => ({
  useRunVerification: () => useRunVerificationMock(),
}));

import { RunVerificationButton } from "./run-verification-button";

afterEach(() => {
  cleanup();
  mutateMock.mockReset();
  useRunVerificationMock.mockReset();
});

describe("RunVerificationButton", () => {
  it("is disabled when there is no worktree", () => {
    useRunVerificationMock.mockReturnValue({ mutate: mutateMock, isPending: false });
    render(<RunVerificationButton taskId="task-1" hasWorktree={false} />);
    const button = screen.getByRole("button", { name: /Run verification/i });
    expect(button).toBeDisabled();
  });

  it("is enabled when a worktree exists and not pending", () => {
    useRunVerificationMock.mockReturnValue({ mutate: mutateMock, isPending: false });
    render(<RunVerificationButton taskId="task-1" hasWorktree />);
    expect(screen.getByRole("button", { name: /Run verification/i })).toBeEnabled();
  });

  it("is disabled and shows a spinner while a run is pending", () => {
    useRunVerificationMock.mockReturnValue({ mutate: mutateMock, isPending: true });
    const { container } = render(
      <RunVerificationButton taskId="task-1" hasWorktree />,
    );
    expect(screen.getByRole("button", { name: /Run verification/i })).toBeDisabled();
    // lucide-react renders inline SVGs; the spinner uses the animate-spin utility class.
    expect(container.querySelector(".animate-spin")).not.toBeNull();
  });

  it("invokes the run mutation with the taskId on click", () => {
    useRunVerificationMock.mockReturnValue({ mutate: mutateMock, isPending: false });
    render(<RunVerificationButton taskId="task-42" hasWorktree />);
    fireEvent.click(screen.getByRole("button", { name: /Run verification/i }));
    expect(mutateMock).toHaveBeenCalledTimes(1);
    expect(mutateMock).toHaveBeenCalledWith("task-42");
  });
});
