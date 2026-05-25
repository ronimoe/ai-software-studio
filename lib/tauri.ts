import { isTauri } from "@tauri-apps/api/core";
import { commands } from "./bindings";
import {
  mockEngines,
  mockProjects,
  mockTasks,
  mockVerification,
} from "./mock-data";
import { sleep } from "./utils";
import type { AppError, Result } from "./bindings";

type Commands = typeof commands;

const mockImpl: Commands = {
  listProjects: async (): Promise<Result<typeof mockProjects, AppError>> => {
    await sleep(50);
    return { status: "ok", data: mockProjects };
  },
  listTasks: async (projectId: string): Promise<Result<typeof mockTasks, AppError>> => {
    await sleep(50);
    return { status: "ok", data: mockTasks.filter((t) => t.projectId === projectId) };
  },
  getTask: async (taskId: string) => {
    await sleep(50);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return {
        status: "error",
        error: {
          code: "notFound",
          message: `task ${taskId} not found`,
          details: null,
        } satisfies AppError,
      };
    }
    return { status: "ok", data: task };
  },
  createTask: async (request) => {
    await sleep(80);
    return {
      status: "ok" as const,
      data: {
        id: `task-${Date.now()}`,
        projectId: request.projectId,
        title: request.title,
        description: request.description,
        outOfScope: request.outOfScope,
        filesToTouchHint: request.filesToTouchHint,
        acceptanceCriteria: request.acceptanceCriteria.map((label, i) => ({
          id: `ac-${i}`,
          label,
          satisfied: false,
        })),
        constraints: request.constraints,
        selectedEngine: request.selectedEngine,
        status: "draft" as const,
        risk: "unknown" as const,
        branchName: null,
        worktreePath: null,
        createdAt: new Date().toISOString(),
      },
    };
  },
  openProject: async (path: string) => {
    await sleep(80);
    return {
      status: "ok" as const,
      data: {
        id: `proj-${Date.now()}`,
        name: path.split("/").pop() ?? "repo",
        path,
        defaultBranch: "main",
      },
    };
  },
  listEngines: async () => {
    await sleep(50);
    return { status: "ok", data: mockEngines };
  },
  detectEngines: async () => {
    await sleep(80);
    return { status: "ok", data: mockEngines };
  },
  listVerification: async (taskId: string) => {
    await sleep(50);
    return { status: "ok", data: mockVerification.filter((v) => v.taskId === taskId) };
  },
  createWorktree: async (taskId: string) => {
    await sleep(120);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return {
        status: "error" as const,
        error: { code: "notFound" as const, message: `task ${taskId} not found`, details: null },
      };
    }
    return {
      status: "ok" as const,
      data: {
        ...task,
        status: "worktreeCreated" as const,
        branchName: `aistudio/task-${taskId.replace("task-", "").slice(0, 8)}`,
        worktreePath: `/mock/worktree/${taskId}`,
      },
    };
  },
  removeWorktree: async (_taskId: string) => {
    await sleep(80);
    return { status: "ok" as const, data: null };
  },
  startTask: async (taskId: string) => {
    await sleep(80);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return {
        status: "error" as const,
        error: { code: "notFound" as const, message: `task ${taskId} not found`, details: null },
      };
    }
    return { status: "ok" as const, data: { ...task, status: "running" as const } };
  },
  stopTask: async (taskId: string) => {
    await sleep(80);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return {
        status: "error" as const,
        error: { code: "notFound" as const, message: `task ${taskId} not found`, details: null },
      };
    }
    return { status: "ok" as const, data: { ...task, status: "stopped" as const } };
  },
  getRunStatus: async (taskId: string) => {
    await sleep(40);
    return { status: "ok" as const, data: { taskId, running: false } };
  },
};

function pickImpl(): Commands {
  if (typeof window !== "undefined" && isTauri()) {
    return commands;
  }
  // Dev-mode: fall back to mocks.
  return mockImpl;
}

const impl = pickImpl();

export const tauri: Commands = impl;
export type { Project, Task, EngineStatus, VerificationRun, AppError, Result } from "./bindings";
