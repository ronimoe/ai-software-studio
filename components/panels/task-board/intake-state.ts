export interface IntakeForm {
  title: string;
  description: string;
  acceptanceCriteria: string; // one per line
  constraints: string;        // one per line
  outOfScope: string;
  filesToTouchHint: string;
}

export const STEPS = [
  { id: "title", label: "Title & description" },
  { id: "acceptance", label: "Acceptance criteria" },
  { id: "constraints", label: "Constraints" },
  { id: "out-of-scope", label: "Out of scope" },
  { id: "files", label: "Files to touch" },
  { id: "review", label: "Review" },
] as const;

export type StepId = (typeof STEPS)[number]["id"];

export const emptyForm: IntakeForm = {
  title: "",
  description: "",
  acceptanceCriteria: "",
  constraints: "",
  outOfScope: "",
  filesToTouchHint: "",
};

export function isStepValid(step: StepId, form: IntakeForm): boolean {
  switch (step) {
    case "title":
      return form.title.trim().length > 0;
    case "acceptance":
      return splitLines(form.acceptanceCriteria).length > 0;
    case "constraints":
    case "out-of-scope":
    case "files":
    case "review":
      return true;
  }
}

export function splitLines(s: string): string[] {
  return s
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
}
