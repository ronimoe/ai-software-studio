import { describe, it, expect } from "vitest";
import { tauri } from "./tauri";

describe("tauri bridge (dev-mode mock dispatcher)", () => {
  it("listProjects returns the default project", async () => {
    const result = await tauri.listProjects();
    expect(result.status).toBe("ok");
    if (result.status !== "ok") return;
    expect(result.data).toHaveLength(1);
    expect(result.data[0].id).toBe("proj-default");
  });

  it("listTasks filters by projectId", async () => {
    const result = await tauri.listTasks("proj-default");
    expect(result.status).toBe("ok");
    if (result.status !== "ok") return;
    expect(result.data.length).toBeGreaterThan(0);
    expect(result.data.every((t) => t.projectId === "proj-default")).toBe(true);
  });

  it("getTask returns error Result on miss", async () => {
    const result = await tauri.getTask("nope");
    expect(result.status).toBe("error");
    if (result.status !== "error") return;
    expect(result.error).toMatchObject({ code: "notFound" });
  });

  it("listEngines returns both Claude Code and Codex CLI", async () => {
    const result = await tauri.listEngines();
    expect(result.status).toBe("ok");
    if (result.status !== "ok") return;
    expect(result.data.map((e) => e.id).sort()).toEqual(["claude-code", "codex-cli"]);
  });
});
