import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";

interface PanelFrameProps {
  title: string;
  subtitle?: string;
  badge?: string;
  actions?: ReactNode;
  className?: string;
  bodyClassName?: string;
  children: ReactNode;
}

export function PanelFrame({
  title,
  subtitle,
  badge,
  actions,
  className,
  bodyClassName,
  children,
}: PanelFrameProps) {
  return (
    <section className={cn("panel-surface flex flex-col overflow-hidden", className)}>
      <header className="flex items-center justify-between gap-3 border-b border-border/60 px-4 py-3">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-medium text-foreground">{title}</h2>
          {badge && (
            <Badge variant="secondary" className="text-[10px] uppercase tracking-wider">
              {badge}
            </Badge>
          )}
        </div>
        {actions && <div className="flex items-center gap-1">{actions}</div>}
      </header>
      {subtitle && (
        <p className="border-b border-border/60 px-4 py-2 text-xs text-muted-foreground">{subtitle}</p>
      )}
      <div className={cn("flex-1 overflow-auto p-4", bodyClassName)}>{children}</div>
    </section>
  );
}
