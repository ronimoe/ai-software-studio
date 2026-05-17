"use client";

import { DashboardShell } from "@/components/layout/dashboard-shell";
import { PanelFrame } from "@/components/layout/panel-frame";

function Placeholder({ name }: { name: string }) {
  return (
    <PanelFrame title={name} badge="WIP">
      <p className="text-sm text-muted-foreground">Panel content arrives in the next task.</p>
    </PanelFrame>
  );
}

export default function HomePage() {
  return (
    <DashboardShell
      left={
        <>
          <Placeholder name="Task Board" />
          <Placeholder name="Engineering Snapshot" />
        </>
      }
      center={
        <>
          <Placeholder name="Agent Workspace" />
          <Placeholder name="Review Room" />
        </>
      }
      right={
        <>
          <Placeholder name="Context Graph" />
          <Placeholder name="Conversation" />
          <Placeholder name="Agent Manager" />
        </>
      }
    />
  );
}
