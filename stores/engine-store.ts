import { create } from "zustand";

interface EngineUiState {
  preferredEngineId: string;
  setPreferredEngine: (id: string) => void;
}

export const useEngineStore = create<EngineUiState>((set) => ({
  preferredEngineId: "claude-code",
  setPreferredEngine: (id) => set({ preferredEngineId: id }),
}));
