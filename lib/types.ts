// UI-only types — NOT crossing the Tauri boundary.
// Boundary types live in lib/bindings.ts (generated from Rust).

export type ConversationAuthor = "user" | "agent" | "system";

export interface ConversationMessage {
  id: string;
  author: ConversationAuthor;
  authorName: string;
  body: string;
  timestamp: string;
}

export interface ContextGraphNode {
  id: string;
  label: string;
  kind: "task" | "engine" | "file" | "branch";
  x: number;
  y: number;
}

export interface ContextGraphEdge {
  from: string;
  to: string;
}

export interface SnapshotMetric {
  label: string;
  value: string;
  trend?: "up" | "down" | "flat";
}

export interface ActiveAgent {
  engineId: string;
  taskId: string;
  status: "running" | "idle" | "blocked";
}
