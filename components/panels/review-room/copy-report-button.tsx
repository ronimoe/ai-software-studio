"use client";

import { useState } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Loader2, Clipboard, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useRenderPrReport } from "@/features/pr/use-render-pr-report";

interface Props {
  taskId: string;
}

export function CopyReportButton({ taskId }: Props) {
  const render = useRenderPrReport();
  const [copied, setCopied] = useState(false);

  const onClick = async () => {
    const md = await render.mutateAsync(taskId);
    await writeText(md);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <Button size="sm" variant="ghost" onClick={onClick} disabled={render.isPending}>
      {render.isPending ? (
        <Loader2 className="mr-1 h-3 w-3 animate-spin" />
      ) : copied ? (
        <Check className="mr-1 h-3 w-3 text-emerald-500" />
      ) : (
        <Clipboard className="mr-1 h-3 w-3" />
      )}
      Copy PR report
    </Button>
  );
}
