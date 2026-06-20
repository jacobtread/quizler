import { MAX_ANSWER_LENGTH } from "$lib/constants";
import { z } from "zod";

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
