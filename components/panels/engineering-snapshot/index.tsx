"use client";

import { ArrowDown, ArrowUp, Minus } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { mockSnapshot } from "@/lib/mock-data";

const trendIcon = {
  up: <ArrowUp className="h-3 w-3 text-success" />,
  down: <ArrowDown className="h-3 w-3 text-destructive" />,
  flat: <Minus className="h-3 w-3 text-muted-foreground" />,
};

export function EngineeringSnapshotPanel() {
  return (
    <PanelFrame title="Engineering Snapshot" bodyClassName="space-y-2">
      {mockSnapshot.map((m) => (
        <div
          key={m.label}
          className="flex items-center justify-between rounded-md border border-border/40 bg-muted/40 px-3 py-2"
        >
          <div className="flex flex-col">
            <span className="text-[11px] text-muted-foreground">{m.label}</span>
            <span className="text-sm font-medium">{m.value}</span>
          </div>
          {m.trend && trendIcon[m.trend]}
        </div>
      ))}
    </PanelFrame>
  );
}
