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
