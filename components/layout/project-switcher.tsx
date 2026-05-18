"use client";

import { FolderPlus, ChevronDown, Loader2 } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useProjects } from "@/features/projects/use-projects";
import { useOpenProject } from "@/features/projects/use-open-project";
import { useUiStore } from "@/stores/ui-store";

export function ProjectSwitcher() {
  const { data: projects = [], isLoading } = useProjects();
  const openProject = useOpenProject();
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const setActiveProject = useUiStore((s) => s.setActiveProject);
  const active = projects.find((p) => p.id === activeProjectId);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/40 px-2.5 py-1 text-xs font-medium hover:bg-muted/60">
        {(isLoading || openProject.isPending) && (
          <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />
        )}
        <span className="font-mono">
          {openProject.isPending ? "opening…" : (active?.name ?? "no workspace")}
        </span>
        <ChevronDown className="h-3 w-3 text-muted-foreground" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="min-w-56">
        <DropdownMenuLabel className="text-[10px] uppercase tracking-wider text-muted-foreground">
          Workspaces
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        {projects.length === 0 ? (
          <DropdownMenuItem disabled className="text-xs">
            No projects
          </DropdownMenuItem>
        ) : (
          projects.map((p) => (
            <DropdownMenuItem
              key={p.id}
              className="flex flex-col items-start gap-0.5 text-xs"
              onSelect={() => setActiveProject(p.id)}
            >
              <span className="font-medium">{p.name}</span>
              <span className="font-mono text-[10px] text-muted-foreground">{p.path}</span>
            </DropdownMenuItem>
          ))
        )}
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onSelect={(e) => {
            e.preventDefault();
            openProject.mutate();
          }}
          disabled={openProject.isPending}
          className="text-xs"
        >
          {openProject.isPending ? (
            <Loader2 className="mr-2 h-3 w-3 animate-spin" />
          ) : (
            <FolderPlus className="mr-2 h-3 w-3" />
          )}
          <span>{openProject.isPending ? "Opening…" : "Open project…"}</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
