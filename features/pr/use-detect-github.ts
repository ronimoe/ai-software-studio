import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useDetectGithub() {
  return useQuery({
    queryKey: ["github", "detect"],
    queryFn: async () => {
      const result = await tauri.detectGithub();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    staleTime: 60_000,
  });
}
