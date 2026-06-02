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
});
