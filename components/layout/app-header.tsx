"use client";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ProjectSwitcher } from "./project-switcher";
import { ThemeToggle } from "./theme-toggle";
import pkg from "@/package.json";

export function AppHeader() {
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
        <ProjectSwitcher />
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
        <Badge variant="outline" className="hidden text-[10px] sm:inline-flex">v{pkg.version}</Badge>
        <ThemeToggle />
        <Avatar className="h-7 w-7">
          <AvatarFallback className="text-[11px]">R</AvatarFallback>
        </Avatar>
      </div>
    </header>
  );
}
