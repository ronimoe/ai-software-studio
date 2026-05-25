import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";

const writeTextMock = vi.fn();
const mutateAsyncMock = vi.fn();
const useRenderPrReportMock = vi.fn();

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (...args: unknown[]) => writeTextMock(...args),
}));

vi.mock("@/features/pr/use-render-pr-report", () => ({
  useRenderPrReport: () => useRenderPrReportMock(),
}));

import { CopyReportButton } from "./copy-report-button";

afterEach(() => {
  cleanup();
  writeTextMock.mockReset();
  mutateAsyncMock.mockReset();
  useRenderPrReportMock.mockReset();
});

describe("CopyReportButton", () => {
  it("renders the clipboard icon and label when idle", () => {
    useRenderPrReportMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });
    render(<CopyReportButton taskId="task-1" />);
    expect(screen.getByRole("button", { name: /Copy PR report/i })).toBeEnabled();
  });

  it("is disabled and shows a spinner while the render mutation is pending", () => {
    useRenderPrReportMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: true });
    const { container } = render(<CopyReportButton taskId="task-1" />);
    expect(screen.getByRole("button", { name: /Copy PR report/i })).toBeDisabled();
    expect(container.querySelector(".animate-spin")).not.toBeNull();
  });

  it("renders the report and writes it to the clipboard on click", async () => {
    mutateAsyncMock.mockResolvedValue("# Report markdown");
    writeTextMock.mockResolvedValue(undefined);
    useRenderPrReportMock.mockReturnValue({ mutateAsync: mutateAsyncMock, isPending: false });
    render(<CopyReportButton taskId="task-42" />);

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Copy PR report/i }));
    });

    expect(mutateAsyncMock).toHaveBeenCalledWith("task-42");
    await waitFor(() => expect(writeTextMock).toHaveBeenCalledWith("# Report markdown"));
  });
});
