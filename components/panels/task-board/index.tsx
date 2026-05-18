"use client";

import { useState } from "react";
import { Plus } from "lucide-react";
import { PanelFrame } from "@/components/layout/panel-frame";
import { Button } from "@/components/ui/button";
import { useTasks } from "@/features/tasks/use-tasks";
import { useUiStore } from "@/stores/ui-store";
import { TaskCard } from "./task-card";
import { NewTaskDialog } from "./new-task-dialog";

export function TaskBoardPanel() {
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const setActiveTask = useUiStore((s) => s.setActiveTask);
  const { data: tasks = [], isLoading } = useTasks(activeProjectId ?? "");
  const [open, setOpen] = useState(false);

  return (
    <>
      <PanelFrame
        title="Task Board"
        badge="Initiative"
        actions={
          <Button
            size="icon"
            variant="ghost"
            className="h-7 w-7"
            aria-label="Add task"
            onClick={() => setOpen(true)}
            disabled={!activeProjectId}
          >
            <Plus className="h-4 w-4" />
          </Button>
        }
        bodyClassName="space-y-2"
      >
        {!activeProjectId && (
          <p className="text-xs text-muted-foreground">Open a project to start adding tasks.</p>
        )}
        {activeProjectId && isLoading && (
          <p className="text-xs text-muted-foreground">Loading tasks…</p>
        )}
        {activeProjectId && !isLoading && tasks.length === 0 && (
          <p className="text-xs text-muted-foreground">No tasks yet. Click + to create one.</p>
        )}
        {tasks.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            active={task.id === activeTaskId}
            onSelect={() => setActiveTask(task.id)}
          />
        ))}
      </PanelFrame>
      <NewTaskDialog open={open} onOpenChange={setOpen} />
    </>
  );
}
