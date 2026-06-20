import {
  MAX_DESCRIPTION_LENGTH,
  MAX_MAX_PLAYERS,
  MAX_QUESTIONS,
  MAX_TITLE_LENGTH,
  MIN_MAX_PLAYERS
} from "$lib/constants";
import { z } from "zod";
import { questionSchema } from "./question";
import { NameFiltering } from "$api/models";

export const createDataSchema = z.object({
  name: z.string().trim().max(MAX_TITLE_LENGTH),
  text: z.string().trim().max(MAX_DESCRIPTION_LENGTH),
  max_players: z.number().min(MIN_MAX_PLAYERS).max(MAX_MAX_PLAYERS),
  filtering: z.enum(NameFiltering),
  questions: z.array(questionSchema).min(1).max(MAX_QUESTIONS)
});
