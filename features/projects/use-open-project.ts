import { useMutation, useQueryClient } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";
import { tauri } from "@/lib/tauri";
import { useUiStore } from "@/stores/ui-store";

async function pickDirectory(): Promise<string | null> {
  if (isTauri()) {
    const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: "Open a git repository",
    });
    if (!selected || Array.isArray(selected)) return null;
    return selected;
  }
  // Browser dev mode: native dialog plugin is unavailable. Fall back to a
  // prompt so the mock dispatcher in lib/tauri.ts still gets exercised.
  if (typeof window === "undefined") return null;
  return window.prompt("Mock dev mode: enter a git repository path") ?? null;
}

export function useOpenProject() {
  const queryClient = useQueryClient();
  const setActiveProject = useUiStore((s) => s.setActiveProject);

  return useMutation({
    mutationFn: async () => {
      const selected = await pickDirectory();
      if (!selected) return null;

      const result = await tauri.openProject(selected);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (project) => {
      if (!project) return;
      queryClient.invalidateQueries({ queryKey: ["projects"] });
      setActiveProject(project.id);
    },
  });
}
