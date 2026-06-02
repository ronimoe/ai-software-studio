import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function usePauseDispatch() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      const result = await tauri.pauseDispatch();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["dispatch-status"] }),
  });
}

export function useResumeDispatch() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      const result = await tauri.resumeDispatch();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["dispatch-status"] }),
  });
}
