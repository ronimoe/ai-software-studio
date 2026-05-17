"use client";

const bars = [3, 7, 5, 9, 4, 8, 6, 10, 5, 7, 9, 6, 8];

export function EvidenceArtifacts() {
  return (
    <div className="rounded-md border border-border/40 bg-muted/30 p-3">
      <h4 className="mb-2 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
        Evidence Artifacts
      </h4>
      <div className="flex h-12 items-end gap-1">
        {bars.map((h, i) => (
          <div
            key={i}
            style={{ height: `${h * 8}%` }}
            className="w-2 rounded-sm bg-primary/60"
          />
        ))}
      </div>
    </div>
  );
}
