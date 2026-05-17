"use client";

import { cn } from "@/lib/utils";
import { PanelFrame } from "@/components/layout/panel-frame";
import { mockConversation } from "@/lib/mock-data";

const authorColor = {
  user: "border-primary/40 bg-primary/10",
  agent: "border-border/60 bg-muted/40",
  system: "border-warning/40 bg-warning/10",
};

export function ConversationPanel() {
  return (
    <PanelFrame title="Comment / Conversation" badge="Timeline" bodyClassName="space-y-2">
      {mockConversation.map((m) => (
        <div key={m.id} className={cn("rounded-md border p-2.5 text-xs", authorColor[m.author])}>
          <div className="mb-1 flex items-center justify-between text-[10px] text-muted-foreground">
            <span className="font-medium">{m.authorName}</span>
            <span>{m.timestamp}</span>
          </div>
          <p className="leading-relaxed">{m.body}</p>
        </div>
      ))}
    </PanelFrame>
  );
}
