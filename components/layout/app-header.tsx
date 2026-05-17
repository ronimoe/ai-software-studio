"use client";

import { ChevronDown } from "lucide-react";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useProjects } from "@/features/projects/use-projects";
import { useUiStore } from "@/stores/ui-store";
import { ThemeToggle } from "./theme-toggle";

export function AppHeader() {
  const { data: projects = [] } = useProjects();
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const setActiveProject = useUiStore((s) => s.setActiveProject);
  const activeProject = projects.find((p) => p.id === activeProjectId);

  return (
    <header className="flex h-14 items-center justify-between gap-4 border-b border-border/60 bg-background/80 px-5 backdrop-blur">
      <div className="flex items-center gap-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary/20 font-mono text-sm font-semibold text-primary">
          AS
        </div>
        <div className="flex flex-col leading-tight">
          <span className="text-sm font-semibold">AI Software Studio</span>
          <span className="text-[11px] text-muted-foreground">Local-first</span>
        </div>
        <Separator orientation="vertical" className="ml-2 h-6" />
        <DropdownMenu>
          <DropdownMenuTrigger className="flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/40 px-2.5 py-1 text-xs font-medium hover:bg-muted/60">
            <span className="font-mono">{activeProject?.name ?? "no workspace"}</span>
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
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="hidden flex-1 flex-col items-center text-center md:flex">
        <h1 className="text-base font-semibold tracking-tight">The AI-first IDE</h1>
        <p className="text-[11px] text-muted-foreground">
          Watch first. Evidence-first. Human accountable.
        </p>
      </div>

      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1 rounded-md border border-border/60 bg-muted/40 p-0.5 text-xs">
          <button className="rounded-sm bg-card px-2 py-1 font-medium shadow-sm">Studio</button>
          <button className="px-2 py-1 text-muted-foreground hover:text-foreground">Trace</button>
        </div>
        <Separator orientation="vertical" className="h-6" />
        <Badge variant="outline" className="hidden text-[10px] sm:inline-flex">v0.0.1</Badge>
        <ThemeToggle />
        <Avatar className="h-7 w-7">
          <AvatarFallback className="text-[11px]">R</AvatarFallback>
        </Avatar>
      </div>
    </header>
  );
}
