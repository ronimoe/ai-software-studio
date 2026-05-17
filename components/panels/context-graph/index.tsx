"use client";

import { PanelFrame } from "@/components/layout/panel-frame";
import { ActiveAgents } from "./active-agents";
import { GraphSvg } from "./graph-svg";

export function ContextGraphPanel() {
  return (
    <PanelFrame title="Context Graph" badge="Live">
      <div className="space-y-4">
        <ActiveAgents />
        <GraphSvg />
      </div>
    </PanelFrame>
  );
}
