import { useMutation, useQueryClient } from "@tanstack/react-query";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { tauri } from "@/lib/tauri";
import { useUiStore } from "@/stores/ui-store";

export function useOpenProject() {
  const queryClient = useQueryClient();
  const setActiveProject = useUiStore((s) => s.setActiveProject);

  return useMutation({
    mutationFn: async () => {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Open a git repository",
      });
      if (!selected || Array.isArray(selected)) return null;

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
