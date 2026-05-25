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
  runVerification: async (taskId: string) => {
    await sleep(400);
    return {
      status: "ok" as const,
      data: {
        id: `vr-${Date.now()}`,
        taskId,
        startedAt: new Date().toISOString(),
        checks: [
          { kind: "install", status: "passed" as const, durationMs: 800, logExcerpt: "Lockfile up to date" },
          { kind: "typecheck", status: "passed" as const, durationMs: 1100, logExcerpt: null },
          { kind: "lint", status: "warning" as const, durationMs: 400, logExcerpt: "1 warning" },
          { kind: "test", status: "passed" as const, durationMs: 2200, logExcerpt: "42 passed" },
          { kind: "build", status: "passed" as const, durationMs: 3000, logExcerpt: null },
        ],
      },
    };
  },
  getVerificationSettings: async (_projectId: string) => {
    await sleep(20);
    return {
      status: "ok" as const,
      data: {
        install: "pnpm install",
        typecheck: "pnpm typecheck",
        lint: "pnpm lint",
        test: "pnpm test",
        build: "pnpm build",
      },
    };
  },
  setVerificationSettings: async (_projectId: string, _settings) => {
    await sleep(20);
    return { status: "ok" as const, data: null };
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
  getChangedFiles: async (_taskId: string) => {
    await sleep(40);
    return {
      status: "ok" as const,
      data: [
        { path: "src/auth/magic-link.ts", status: "modified" as const, additions: 14, deletions: 3 },
        { path: "src/middleware.ts", status: "modified" as const, additions: 2, deletions: 0 },
        { path: "tests/auth.test.ts", status: "added" as const, additions: 24, deletions: 0 },
      ],
    };
  },
  getFileDiff: async (_taskId: string, path: string) => {
    await sleep(40);
    return {
      status: "ok" as const,
      data: `--- a/${path}\n+++ b/${path}\n@@ -1,3 +1,4 @@\n line one\n-line two\n+line two (changed)\n+line three (new)\n line four\n`,
    };
  },
  reconcileAfterExit: async (taskId: string) => {
    await sleep(40);
    const task = mockTasks.find((t) => t.id === taskId);
    if (!task) {
      return { status: "error" as const, error: { code: "notFound" as const, message: `task ${taskId} not found`, details: null } };
    }
    return { status: "ok" as const, data: { ...task, status: "reviewReady" as const } };
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
