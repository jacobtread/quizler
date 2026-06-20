import { MAX_ANSWER_LENGTH } from "$lib/constants";
import { z } from "zod";

// Different answer types
export const enum AnswerType {
  Single = "Single",
  Multiple = "Multiple",
  TrueFalse = "TrueFalse",
  Typer = "Typer"
}

// Answer schemas for each different type
export type Answer =
  | { ty: AnswerType.Single; answer: number }
  | { ty: AnswerType.Multiple; answers: number[] }
  | { ty: AnswerType.TrueFalse; answer: boolean }
  | { ty: AnswerType.Typer; answer: string };

// Piece of text representing an answer
export const answerText = z
  .string()
  .trim()
  .nonempty("cannot be empty")
  .max(MAX_ANSWER_LENGTH, `cannot be longer than ${MAX_ANSWER_LENGTH}`);

// Schema for question answers
export const answerValueSchema = z.object({
  id: z.number(),
  value: answerText,
  correct: z.boolean()
});

// Answer value type inferred from its schema
export type AnswerValue = z.infer<typeof answerValueSchema>;
