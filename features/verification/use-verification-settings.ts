import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";
import type { VerificationSettings } from "@/lib/bindings";

export function useVerificationSettings(projectId: string | null) {
  return useQuery({
    queryKey: ["verification-settings", projectId],
    queryFn: async () => {
      const result = await tauri.getVerificationSettings(projectId!);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    enabled: !!projectId,
  });
}

export function useSetVerificationSettings(projectId: string | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (settings: VerificationSettings) => {
      const result = await tauri.setVerificationSettings(projectId!, settings);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["verification-settings", projectId] });
    },
  });
}
