import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useDetectEngines() {
  return useQuery({
    queryKey: ["engines", "detect"],
    queryFn: async () => {
      const result = await tauri.detectEngines();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    staleTime: 60_000,
  });
}
