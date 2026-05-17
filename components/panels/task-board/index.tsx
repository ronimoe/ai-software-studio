"use client";

import { Plus } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { Button } from "@/components/ui/button";
import { useTasks } from "@/features/tasks/use-tasks";
import { useUiStore } from "@/stores/ui-store";
import { TaskCard } from "./task-card";

export function TaskBoardPanel() {
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const setActiveTask = useUiStore((s) => s.setActiveTask);
  const { data: tasks = [], isLoading } = useTasks(activeProjectId);

  return (
    <PanelFrame
      title="Task Board"
      badge="Initiative"
      actions={
        <Button size="icon" variant="ghost" className="h-7 w-7" aria-label="Add task">
          <Plus className="h-4 w-4" />
        </Button>
      }
      bodyClassName="space-y-2"
    >
      {isLoading && <p className="text-xs text-muted-foreground">Loading tasks…</p>}
      {tasks.map((task) => (
        <TaskCard
          key={task.id}
          task={task}
          active={task.id === activeTaskId}
          onSelect={() => setActiveTask(task.id)}
        />
      ))}
    </PanelFrame>
  );
}
