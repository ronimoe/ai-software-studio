import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

const useFileDiffMock = vi.fn();

vi.mock("@/features/diffs/use-file-diff", () => ({
  useFileDiff: (...args: unknown[]) => useFileDiffMock(...args),
}));

vi.mock("react-diff-view/style/index.css", () => ({}));

import { DiffViewerDialog } from "./diff-viewer-dialog";

afterEach(() => {
  cleanup();
  useFileDiffMock.mockReset();
});

describe("DiffViewerDialog", () => {
  it("renders nothing when path is null", () => {
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<DiffViewerDialog taskId="task-1" path={null} onClose={() => {}} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("shows the file path in the title when open", () => {
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<DiffViewerDialog taskId="task-1" path="src/a.ts" onClose={() => {}} />);
    expect(screen.getByText("src/a.ts")).toBeInTheDocument();
  });

  it("shows loading state while the diff is fetching", () => {
    useFileDiffMock.mockReturnValue({ data: undefined, isLoading: true });
    render(<DiffViewerDialog taskId="task-1" path="src/a.ts" onClose={() => {}} />);
    expect(screen.getByText(/Loading diff/i)).toBeInTheDocument();
  });

  it("shows empty state when diff text is blank", () => {
    useFileDiffMock.mockReturnValue({ data: "", isLoading: false });
    render(<DiffViewerDialog taskId="task-1" path="src/a.ts" onClose={() => {}} />);
    expect(screen.getByText(/No diff to show/i)).toBeInTheDocument();
  });

  it("renders parsed diff hunks when diff text is non-empty", () => {
    const patch = [
      "--- a/src/a.ts",
      "+++ b/src/a.ts",
      "@@ -1,3 +1,4 @@",
      " line one",
      "-line two",
      "+line two changed",
      "+line three",
      " line four",
      "",
    ].join("\n");
    useFileDiffMock.mockReturnValue({ data: patch, isLoading: false });
    render(<DiffViewerDialog taskId="task-1" path="src/a.ts" onClose={() => {}} />);
    // Empty state must NOT be shown when the diff parsed into a hunk.
    expect(screen.queryByText(/No diff to show/i)).not.toBeInTheDocument();
    // The added/changed line content from the patch should appear in the rendered hunk.
    expect(screen.getByText(/line two changed/)).toBeInTheDocument();
  });
});
