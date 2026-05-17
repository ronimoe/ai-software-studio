import { create } from "zustand";

interface UiState {
  activeProjectId: string;
  activeTaskId: string | null;
  agentManagerOpen: boolean;
  setActiveTask: (taskId: string | null) => void;
  setActiveProject: (projectId: string) => void;
  toggleAgentManager: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  activeProjectId: "proj-default",
  activeTaskId: "task-042",
  agentManagerOpen: false,
  setActiveTask: (taskId) => set({ activeTaskId: taskId }),
  setActiveProject: (projectId) => set({ activeProjectId: projectId, activeTaskId: null }),
  toggleAgentManager: () => set((s) => ({ agentManagerOpen: !s.agentManagerOpen })),
}));
