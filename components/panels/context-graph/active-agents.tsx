"use client";

import { Badge } from "@/components/ui/badge";
import { useEngines } from "@/features/engines/use-engines";
import { mockActiveAgents } from "@/lib/mock-data";

export function ActiveAgents() {
  const { data: engines = [] } = useEngines();
  return (
    <div className="space-y-1.5">
      <h4 className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
        Active Agents
      </h4>
      {engines.map((e) => {
        const agent = mockActiveAgents.find((a) => a.engineId === e.id);
        return (
          <div
            key={e.id}
            className="flex items-center justify-between rounded-md border border-border/40 bg-muted/30 px-3 py-2 text-xs"
          >
            <div className="flex flex-col">
              <span className="font-medium">{e.name}</span>
              <span className="text-[10px] text-muted-foreground">
                {e.version ? `v${e.version}` : "no version"}
              </span>
            </div>
            <Badge
              variant={e.status === "ready" ? "default" : "outline"}
              className="text-[10px]"
            >
              {agent ? agent.status : e.status}
            </Badge>
          </div>
        );
      })}
    </div>
  );
}
