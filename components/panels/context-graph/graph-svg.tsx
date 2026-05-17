"use client";

import { useContextGraph } from "@/features/context-graph/use-context-graph";
import { useUiStore } from "@/stores/ui-store";

const colorFor: Record<string, string> = {
  task: "var(--primary)",
  engine: "var(--accent)",
  branch: "var(--secondary)",
  file: "var(--muted-foreground)",
};

export function GraphSvg() {
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const { data } = useContextGraph(activeTaskId);
  const nodes = data?.nodes ?? [];
  const edges = data?.edges ?? [];
  return (
    <svg viewBox="0 0 380 300" className="h-44 w-full">
      {edges.map((e, i) => {
        const from = nodes.find((n) => n.id === e.from);
        const to = nodes.find((n) => n.id === e.to);
        if (!from || !to) return null;
        return (
          <line
            key={i}
            x1={from.x}
            y1={from.y}
            x2={to.x}
            y2={to.y}
            stroke="var(--border)"
            strokeWidth={1}
          />
        );
      })}
      {nodes.map((n) => (
        <g key={n.id} transform={`translate(${n.x}, ${n.y})`}>
          <circle r={10} fill={colorFor[n.kind] ?? "var(--muted)"} opacity={0.85} />
          <text
            y={22}
            textAnchor="middle"
            fontSize="9"
            className="fill-foreground text-[9px]"
          >
            {n.label}
          </text>
        </g>
      ))}
    </svg>
  );
}
