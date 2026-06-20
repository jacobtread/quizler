import { MAX_ANSWERS, MAX_QUESTION_LENGTH } from "$lib/constants";
import { z } from "zod";
import { answerText, answerValueSchema } from "./answer";
import { ImageFit, QuestionType } from "$api/models";

// Single choice questions
const singleQuestionSchema = z.object({
  ty: z.literal(QuestionType.Single),
  answers: z
    .array(answerValueSchema)
    .min(1, "Must provide at least one answer")
    .max(MAX_ANSWERS, `Too many answers maximum allowed is ${MAX_ANSWERS}`)
});

// Multiple choice questions
const multipleChoiceQuestionSchema = z.object({
  ty: z.literal(QuestionType.Multiple),
  answers: z
    .array(answerValueSchema)
    .min(1, "Must provide at least one answer")
    .max(MAX_ANSWERS, `Too many answers maximum allowed is ${MAX_ANSWERS}`),
  correct_answers: z.number()
});

// True / False choice questions
const trueFalseQuestionSchema = z.object({
  ty: z.literal(QuestionType.TrueFalse),
  answer: z.boolean()
});

// Typing question
const typingQuestionSchema = z.object({
  ty: z.literal(QuestionType.Typer),
  answers: z
    .array(answerText)
    .min(1, "Must provide at least one answer")
    .max(MAX_ANSWERS, `Too many answers maximum allowed is ${MAX_ANSWERS}`),
  ignore_case: z.boolean()
});

// Schema of question image
const questionImageSchema = z.object({
  uuid: z.uuid(),
  fit: z.enum(ImageFit)
});

// Schema for question scoring
const questionScoringSchema = z.object({
  min_score: z.number(),
  max_score: z.number(),
  bonus_score: z.number()
});

// Base schema shared by all questions regardless of type
const baseQuestionSchema = z.object({
  text: z
    .string()
    .trim()
    .nonempty("Question cannot be empty")
    .max(MAX_QUESTION_LENGTH),
  image: questionImageSchema.nullable(),
  answer_time: z.number(),
  bonus_score_time: z.number(),
  scoring: questionScoringSchema
});

// Type specific question schema portion
const questionTypeSchema = z.discriminatedUnion("ty", [
  singleQuestionSchema,
  multipleChoiceQuestionSchema,
  trueFalseQuestionSchema,
  typingQuestionSchema
]);

// Full schema for questions
export const questionSchema = baseQuestionSchema.and(questionTypeSchema);
