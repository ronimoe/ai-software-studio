"use client";

import { useState } from "react";
import { ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { useCreateTask } from "@/features/tasks/use-create-task";
import { useUiStore } from "@/stores/ui-store";
import {
  emptyForm,
  isStepValid,
  splitLines,
  STEPS,
  type IntakeForm,
  type StepId,
} from "./intake-state";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewTaskDialog({ open, onOpenChange }: Props) {
  const activeProjectId = useUiStore((s) => s.activeProjectId);
  const setActiveTask = useUiStore((s) => s.setActiveTask);
  const createTask = useCreateTask();

  const [form, setForm] = useState<IntakeForm>(emptyForm);
  const [stepIndex, setStepIndex] = useState(0);
  const stepId: StepId = STEPS[stepIndex].id;

  const reset = () => { setForm(emptyForm); setStepIndex(0); };

  const close = () => {
    onOpenChange(false);
    setTimeout(reset, 200);
  };

  const next = () => {
    if (stepIndex < STEPS.length - 1) setStepIndex(stepIndex + 1);
  };

  const back = () => {
    if (stepIndex > 0) setStepIndex(stepIndex - 1);
  };

  const submit = async () => {
    if (!activeProjectId) return;
    const result = await createTask.mutateAsync({
      projectId: activeProjectId,
      title: form.title.trim(),
      description: form.description.trim(),
      acceptanceCriteria: splitLines(form.acceptanceCriteria),
      constraints: splitLines(form.constraints),
      outOfScope: form.outOfScope.trim(),
      filesToTouchHint: form.filesToTouchHint.trim(),
      selectedEngine: null,
    });
    setActiveTask(result.id);
    close();
  };

  const setField = <K extends keyof IntakeForm>(k: K, v: IntakeForm[K]) =>
    setForm((f) => ({ ...f, [k]: v }));

  return (
    <Dialog open={open} onOpenChange={(o) => (o ? onOpenChange(true) : close())}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>New task</DialogTitle>
          <DialogDescription>
            Step {stepIndex + 1} of {STEPS.length} — {STEPS[stepIndex].label}
          </DialogDescription>
        </DialogHeader>

        <div className="min-h-[220px] space-y-3 py-2">
          {stepId === "title" && (
            <>
              <div className="space-y-1.5">
                <Label htmlFor="title">Title</Label>
                <Input
                  id="title"
                  value={form.title}
                  onChange={(e) => setField("title", e.target.value)}
                  placeholder="Add magic link login while preserving JWT flow"
                  autoFocus
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  rows={5}
                  value={form.description}
                  onChange={(e) => setField("description", e.target.value)}
                  placeholder="What is this task and why is it needed?"
                />
              </div>
            </>
          )}

          {stepId === "acceptance" && (
            <div className="space-y-1.5">
              <Label htmlFor="acceptance">Acceptance criteria (one per line)</Label>
              <Textarea
                id="acceptance"
                rows={8}
                value={form.acceptanceCriteria}
                onChange={(e) => setField("acceptanceCriteria", e.target.value)}
                placeholder={"Magic link email delivered in dev\nExisting JWT routes still pass\nSession cookie behavior unchanged"}
                autoFocus
              />
              <p className="text-xs text-muted-foreground">
                At least one criterion required.
              </p>
            </div>
          )}

          {stepId === "constraints" && (
            <div className="space-y-1.5">
              <Label htmlFor="constraints">Constraints (one per line)</Label>
              <Textarea
                id="constraints"
                rows={8}
                value={form.constraints}
                onChange={(e) => setField("constraints", e.target.value)}
                placeholder={"No new external dependencies\nDo not modify src/billing"}
                autoFocus
              />
            </div>
          )}

          {stepId === "out-of-scope" && (
            <div className="space-y-1.5">
              <Label htmlFor="oos">Out of scope</Label>
              <Textarea
                id="oos"
                rows={6}
                value={form.outOfScope}
                onChange={(e) => setField("outOfScope", e.target.value)}
                placeholder="Things the agent should NOT do as part of this task."
                autoFocus
              />
            </div>
          )}

          {stepId === "files" && (
            <div className="space-y-1.5">
              <Label htmlFor="files">Files to touch (hint)</Label>
              <Textarea
                id="files"
                rows={6}
                value={form.filesToTouchHint}
                onChange={(e) => setField("filesToTouchHint", e.target.value)}
                placeholder="src/auth/**, src/middleware/jwt.ts"
                autoFocus
              />
            </div>
          )}

          {stepId === "review" && (
            <div className="space-y-3 text-sm">
              <Row label="Title" value={form.title} />
              <Row label="Description" value={form.description || "—"} multiline />
              <Row
                label="Acceptance criteria"
                value={splitLines(form.acceptanceCriteria).map((l) => `• ${l}`).join("\n") || "—"}
                multiline
              />
              <Row
                label="Constraints"
                value={splitLines(form.constraints).map((l) => `• ${l}`).join("\n") || "—"}
                multiline
              />
              <Row label="Out of scope" value={form.outOfScope || "—"} multiline />
              <Row label="Files to touch" value={form.filesToTouchHint || "—"} multiline />
            </div>
          )}
        </div>

        <div className="flex items-center justify-between pt-3">
          <Button variant="ghost" size="sm" onClick={back} disabled={stepIndex === 0}>
            <ChevronLeft className="mr-1 h-3 w-3" /> Back
          </Button>
          {stepId !== "review" ? (
            <Button size="sm" onClick={next} disabled={!isStepValid(stepId, form)}>
              Next <ChevronRight className="ml-1 h-3 w-3" />
            </Button>
          ) : (
            <Button
              size="sm"
              onClick={submit}
              disabled={!activeProjectId || createTask.isPending || !form.title.trim()}
            >
              {createTask.isPending && <Loader2 className="mr-1 h-3 w-3 animate-spin" />}
              Create task
            </Button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

function Row({ label, value, multiline }: { label: string; value: string; multiline?: boolean }) {
  return (
    <div>
      <div className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground">{label}</div>
      <div className={multiline ? "whitespace-pre-wrap text-sm" : "text-sm"}>{value}</div>
    </div>
  );
}
