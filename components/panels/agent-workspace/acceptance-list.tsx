"use client";

import { Check, Circle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Task } from "@/lib/tauri";

export function AcceptanceList({ items }: { items: Task["acceptanceCriteria"] }) {
  if (items.length === 0) {
    return <p className="text-xs text-muted-foreground">No acceptance criteria.</p>;
  }
  return (
    <ul className="space-y-1.5">
      {items.map((item) => (
        <li key={item.id} className="flex items-start gap-2 text-sm">
          {item.satisfied ? (
            <Check className="mt-0.5 h-4 w-4 shrink-0 text-success" />
          ) : (
            <Circle className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
          )}
          <span className={cn(item.satisfied && "text-muted-foreground line-through")}>{item.label}</span>
        </li>
      ))}
    </ul>
  );
}
