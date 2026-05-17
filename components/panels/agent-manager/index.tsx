"use client";

import { Power } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useEngines } from "@/features/engines/use-engines";

const statusBadge: Record<string, string> = {
  ready: "bg-success/20 text-success",
  notInstalled: "bg-muted text-muted-foreground",
  notAuthenticated: "bg-warning/20 text-warning",
  detected: "bg-primary/20 text-primary",
  error: "bg-destructive/20 text-destructive",
};

export function AgentManagerPanel() {
  const { data: engines = [] } = useEngines();
  return (
    <PanelFrame
      title="Agent Manager"
      badge="Engines"
      actions={
        <Button size="sm" variant="ghost" className="h-7">
          <Power className="mr-1 h-3 w-3" /> Re-detect
        </Button>
      }
      bodyClassName="space-y-2"
    >
      {engines.map((e) => (
        <div
          key={e.id}
          className="flex items-center justify-between rounded-md border border-border/40 bg-muted/30 px-3 py-2"
        >
          <div className="flex flex-col">
            <span className="text-sm font-medium">{e.name}</span>
            <span className="text-[10px] text-muted-foreground">
              {e.binaryPath ?? "binary not found"}
            </span>
          </div>
          <Badge className={`text-[10px] ${statusBadge[e.status] ?? "bg-muted"}`}>
            {e.status}
          </Badge>
        </div>
      ))}
    </PanelFrame>
  );
}
