import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { events } from "@/lib/bindings";

export function useDispatchEvents() {
  const queryClient = useQueryClient();
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    (async () => {
      try {
        unlisten = await events.dispatchEvent.listen(() => {
          queryClient.invalidateQueries({ queryKey: ["dispatch-status"] });
          queryClient.invalidateQueries({ queryKey: ["tasks"] });
        });
      } catch {
        // Dev mode without Tauri runtime — no event stream, fine.
      }
    })();
    return () => unlisten?.();
  }, [queryClient]);
}
