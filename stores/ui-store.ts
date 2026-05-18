import { create } from "zustand";

interface UiState {
  activeProjectId: string | null;
  activeTaskId: string | null;
  agentManagerOpen: boolean;
  setActiveTask: (taskId: string | null) => void;
  setActiveProject: (projectId: string | null) => void;
  toggleAgentManager: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  activeProjectId: null,
  activeTaskId: null,
  agentManagerOpen: false,
  setActiveTask: (taskId) => set({ activeTaskId: taskId }),
  setActiveProject: (projectId) => set({ activeProjectId: projectId, activeTaskId: null }),
  toggleAgentManager: () => set((s) => ({ agentManagerOpen: !s.agentManagerOpen })),
}));
