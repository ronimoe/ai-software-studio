import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useDispatchStatus() {
  return useQuery({
    queryKey: ["dispatch-status"],
    queryFn: async () => {
      const result = await tauri.getDispatchStatus();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  });
}
