import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

const mutateAsyncMock = vi.fn();
const useDetectGithubMock = vi.fn();
const useCreatePrMock = vi.fn();

vi.mock("@/features/pr/use-detect-github", () => ({
  useDetectGithub: () => useDetectGithubMock(),
}));

vi.mock("@/features/pr/use-create-pr", () => ({
  useCreatePr: () => useCreatePrMock(),
}));

import { CreatePrButton } from "./create-pr-button";

const authed = { auth: "authed" as const, binaryPath: "/gh", account: "u" };
const notAuthed = { auth: "notAuthed" as const, binaryPath: "/gh", account: null };
const notInstalled = { auth: "notInstalled" as const, binaryPath: null, account: null };

afterEach(() => {
  cleanup();
  mutateAsyncMock.mockReset();
  useDetectGithubMock.mockReset();
  useCreatePrMock.mockReset();
});

describe("CreatePrButton", () => {
  it("is disabled when there is no worktree", () => {
    useDetectGithubMock.mockReturnValue({ data: authed });
    useCreatePrMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });
    render(<CreatePrButton taskId="t1" hasWorktree={false} />);
    expect(screen.getByRole("button", { name: /Create PR/i })).toBeDisabled();
  });

  it("is disabled when gh is not authed", () => {
    useDetectGithubMock.mockReturnValue({ data: notAuthed });
    useCreatePrMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });
    render(<CreatePrButton taskId="t1" hasWorktree />);
    expect(screen.getByRole("button", { name: /Create PR/i })).toBeDisabled();
  });

  it("is disabled when gh is not installed", () => {
    useDetectGithubMock.mockReturnValue({ data: notInstalled });
    useCreatePrMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });
    render(<CreatePrButton taskId="t1" hasWorktree />);
    expect(screen.getByRole("button", { name: /Create PR/i })).toBeDisabled();
  });

  it("is enabled when authed and worktree exists", () => {
    useDetectGithubMock.mockReturnValue({ data: authed });
    useCreatePrMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });
    render(<CreatePrButton taskId="t1" hasWorktree />);
    expect(screen.getByRole("button", { name: /Create PR/i })).toBeEnabled();
  });

  it("shows the spinner while pending and is disabled", () => {
    useDetectGithubMock.mockReturnValue({ data: authed });
    useCreatePrMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: true });
    const { container } = render(<CreatePrButton taskId="t1" hasWorktree />);
    expect(screen.getByRole("button", { name: /Create PR/i })).toBeDisabled();
    expect(container.querySelector(".animate-spin")).not.toBeNull();
  });

  it("invokes createPr with non-draft request and transforms into an Open PR link on success", async () => {
    mutateAsyncMock.mockResolvedValue({
      url: "https://github.com/owner/repo/pull/7",
      branch: "aistudio/task-t1",
      base: "main",
    });
    useDetectGithubMock.mockReturnValue({ data: authed });
    useCreatePrMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });

    render(<CreatePrButton taskId="t1" hasWorktree />);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Create PR/i }));
    });

    expect(mutateAsyncMock).toHaveBeenCalledWith({ taskId: "t1", baseBranch: null, draft: false });
    await waitFor(() => {
      const link = screen.getByRole("link", { name: /Open PR/i });
      expect(link).toHaveAttribute("href", "https://github.com/owner/repo/pull/7");
    });
  });
});
