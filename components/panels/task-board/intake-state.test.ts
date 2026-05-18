import { describe, it, expect } from "vitest";
import { emptyForm, isStepValid, splitLines, type IntakeForm } from "./intake-state";

describe("isStepValid - title", () => {
  it("returns false for empty form (no title)", () => {
    expect(isStepValid("title", emptyForm)).toBe(false);
  });

  it("returns false when title is whitespace-only", () => {
    const form: IntakeForm = { ...emptyForm, title: "   " };
    expect(isStepValid("title", form)).toBe(false);
  });

  it("returns true when title has content", () => {
    const form: IntakeForm = { ...emptyForm, title: "Build a feature" };
    expect(isStepValid("title", form)).toBe(true);
  });
});

describe("isStepValid - acceptance", () => {
  it("returns false when acceptanceCriteria is empty", () => {
    expect(isStepValid("acceptance", emptyForm)).toBe(false);
  });

  it("returns false when acceptanceCriteria is only whitespace lines", () => {
    const form: IntakeForm = {
      ...emptyForm,
      acceptanceCriteria: "   \n\t\n  ",
    };
    expect(isStepValid("acceptance", form)).toBe(false);
  });

  it("returns true when at least one non-blank line exists", () => {
    const form: IntakeForm = {
      ...emptyForm,
      acceptanceCriteria: "\nUser can submit form\n",
    };
    expect(isStepValid("acceptance", form)).toBe(true);
  });
});

describe("isStepValid - optional steps", () => {
  it("returns true for constraints regardless of form content", () => {
    expect(isStepValid("constraints", emptyForm)).toBe(true);
    const form: IntakeForm = { ...emptyForm, constraints: "no external APIs" };
    expect(isStepValid("constraints", form)).toBe(true);
  });

  it("returns true for out-of-scope regardless of form content", () => {
    expect(isStepValid("out-of-scope", emptyForm)).toBe(true);
    const form: IntakeForm = { ...emptyForm, outOfScope: "auth flow" };
    expect(isStepValid("out-of-scope", form)).toBe(true);
  });

  it("returns true for files regardless of form content", () => {
    expect(isStepValid("files", emptyForm)).toBe(true);
    const form: IntakeForm = { ...emptyForm, filesToTouchHint: "src/foo.ts" };
    expect(isStepValid("files", form)).toBe(true);
  });

  it("returns true for review regardless of form content", () => {
    expect(isStepValid("review", emptyForm)).toBe(true);
    const form: IntakeForm = { ...emptyForm, title: "anything" };
    expect(isStepValid("review", form)).toBe(true);
  });
});

describe("splitLines", () => {
  it("returns an empty array for an empty string", () => {
    expect(splitLines("")).toEqual([]);
  });

  it("returns a single-element array for a single line", () => {
    expect(splitLines("hello")).toEqual(["hello"]);
  });

  it("preserves order across multiple lines", () => {
    expect(splitLines("one\ntwo\nthree")).toEqual(["one", "two", "three"]);
  });

  it("trims each line and drops blank lines", () => {
    expect(splitLines("  one  \n\n  two\n   \nthree  ")).toEqual([
      "one",
      "two",
      "three",
    ]);
  });

  it("handles CRLF input by stripping the trailing \\r via trim", () => {
    expect(splitLines("one\r\ntwo\r\nthree")).toEqual(["one", "two", "three"]);
  });
});

describe("emptyForm", () => {
  it("has all 6 fields as empty strings", () => {
    expect(emptyForm).toEqual({
      title: "",
      description: "",
      acceptanceCriteria: "",
      constraints: "",
      outOfScope: "",
      filesToTouchHint: "",
    });
  });
});
