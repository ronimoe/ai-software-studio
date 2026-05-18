"use client";

import { useEffect } from "react";
import { DashboardShell } from "@/components/layout/dashboard-shell";
import { TaskBoardPanel } from "@/components/panels/task-board";
import { EngineeringSnapshotPanel } from "@/components/panels/engineering-snapshot";
import { AgentWorkspacePanel } from "@/components/panels/agent-workspace";
import { ReviewRoomPanel } from "@/components/panels/review-room";
import { ContextGraphPanel } from "@/components/panels/context-graph";
import { ConversationPanel } from "@/components/panels/conversation";
import { AgentManagerPanel } from "@/components/panels/agent-manager";
import { useProjects } from "@/features/projects/use-projects";
import { useUiStore } from "@/stores/ui-store";

export default function HomePage() {
  const { data: projects = [] } = useProjects();
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const setActiveProject = useUiStore((s) => s.setActiveProject);

  useEffect(() => {
    if (!activeProjectId && projects.length > 0) {
      setActiveProject(projects[0].id);
    }
  }, [activeProjectId, projects, setActiveProject]);

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
