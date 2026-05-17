"use client";

import type { ReactNode } from "react";
import { AppHeader } from "./app-header";

interface DashboardShellProps {
  left: ReactNode;       // Task Board + Engineering Snapshot
  center: ReactNode;     // Agent Workspace + Review Room
  right: ReactNode;      // Context Graph + Conversation + Agent Manager
}

export function DashboardShell({ left, center, right }: DashboardShellProps) {
  return (
    <div className="grid h-screen grid-rows-[auto_1fr] bg-background">
      <AppHeader />
      <div className="grid min-h-0 grid-cols-[280px_1fr_360px] gap-3 p-3">
        <div className="grid min-h-0 grid-rows-[1fr_auto] gap-3">{left}</div>
        <div className="grid min-h-0 grid-rows-[1fr_auto] gap-3">{center}</div>
        <div className="grid min-h-0 grid-rows-[auto_1fr_auto] gap-3">{right}</div>
      </div>
    </div>
  );
}
