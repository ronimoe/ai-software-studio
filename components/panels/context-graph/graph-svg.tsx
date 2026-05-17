"use client";

import { mockGraphEdges, mockGraphNodes } from "@/lib/mock-data";

const colorFor: Record<string, string> = {
  task: "var(--primary)",
  engine: "var(--accent)",
  branch: "var(--secondary)",
  file: "var(--muted-foreground)",
};

export function GraphSvg() {
  return (
    <svg viewBox="0 0 380 300" className="h-44 w-full">
      {mockGraphEdges.map((e, i) => {
        const from = mockGraphNodes.find((n) => n.id === e.from);
        const to = mockGraphNodes.find((n) => n.id === e.to);
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
      {mockGraphNodes.map((n) => (
        <g key={n.id} transform={`translate(${n.x}, ${n.y})`}>
          <circle r={10} fill={colorFor[n.kind] ?? "var(--muted)"} opacity={0.85} />
          <text
            y={22}
            textAnchor="middle"
            className="fill-foreground text-[9px]"
          >
            {n.label}
          </text>
        </g>
      ))}
    </svg>
  );
}
