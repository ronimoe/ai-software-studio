import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useEngines() {
  return useQuery({
    queryKey: ["engines"],
    queryFn: async () => {
      const result = await tauri.listEngines();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  });
}
