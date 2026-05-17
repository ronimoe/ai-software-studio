"use client";

import { DashboardShell } from "@/components/layout/dashboard-shell";
import { TaskBoardPanel } from "@/components/panels/task-board";
import { EngineeringSnapshotPanel } from "@/components/panels/engineering-snapshot";
import { AgentWorkspacePanel } from "@/components/panels/agent-workspace";
import { ReviewRoomPanel } from "@/components/panels/review-room";
import { ContextGraphPanel } from "@/components/panels/context-graph";
import { ConversationPanel } from "@/components/panels/conversation";
import { AgentManagerPanel } from "@/components/panels/agent-manager";

export default function HomePage() {
  return (
    <DashboardShell
      left={
        <>
          <TaskBoardPanel />
          <EngineeringSnapshotPanel />
        </>
      }
      center={
        <>
          <AgentWorkspacePanel />
          <ReviewRoomPanel />
        </>
      }
      right={
        <>
          <ContextGraphPanel />
          <ConversationPanel />
          <AgentManagerPanel />
        </>
      }
    />
  );
}
