import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import type { ChangedFile } from "@/lib/bindings";

const useChangedFilesMock = vi.fn();
const useFileDiffMock = vi.fn();

vi.mock("@/features/diffs/use-changed-files", () => ({
  useChangedFiles: (...args: unknown[]) => useChangedFilesMock(...args),
}));

vi.mock("@/features/diffs/use-file-diff", () => ({
  useFileDiff: (...args: unknown[]) => useFileDiffMock(...args),
}));

// Skip the CSS side-effect import; jsdom can't parse it.
vi.mock("react-diff-view/style/index.css", () => ({}));

import { ChangedFilesPanel } from "./changed-files-panel";

afterEach(() => {
  cleanup();
  useChangedFilesMock.mockReset();
  useFileDiffMock.mockReset();
});

describe("ChangedFilesPanel", () => {
  it("shows loading state", () => {
    useChangedFilesMock.mockReturnValue({ data: undefined, isLoading: true });
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<ChangedFilesPanel taskId="task-1" />);
    expect(screen.getByText(/Loading changes/i)).toBeInTheDocument();
  });

  it("shows empty state when no files", () => {
    useChangedFilesMock.mockReturnValue({ data: [], isLoading: false });
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<ChangedFilesPanel taskId="task-1" />);
    expect(screen.getByText(/No changes yet/i)).toBeInTheDocument();
  });

  it("renders one button per file with path and +adds/-dels", () => {
    const files: ChangedFile[] = [
      { path: "src/a.ts", status: "modified", additions: 5, deletions: 2 },
      { path: "src/b.ts", status: "added", additions: 10, deletions: 0 },
    ];
    useChangedFilesMock.mockReturnValue({ data: files, isLoading: false });
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<ChangedFilesPanel taskId="task-1" />);
    expect(screen.getByText("src/a.ts")).toBeInTheDocument();
    expect(screen.getByText("src/b.ts")).toBeInTheDocument();
    expect(screen.getByText("+5 −2")).toBeInTheDocument();
    expect(screen.getByText("+10 −0")).toBeInTheDocument();
  });

  it("opens the diff viewer dialog when a file is clicked", () => {
    const files: ChangedFile[] = [
      { path: "src/a.ts", status: "modified", additions: 1, deletions: 0 },
    ];
    useChangedFilesMock.mockReturnValue({ data: files, isLoading: false });
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<ChangedFilesPanel taskId="task-1" />);
    // Dialog starts closed: title not in the document.
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /src\/a\.ts/ }));
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });
});
