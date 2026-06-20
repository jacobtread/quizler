import type { SHADOW_ITEM_MARKER_PROPERTY_NAME } from "svelte-dnd-action";
import { z } from "zod";
import type { questionSchema } from "$api/schema/question";

// Question types
export enum QuestionType {
  Single = "Single",
  Multiple = "Multiple",
  TrueFalse = "TrueFalse",
  Typer = "Typer"
}

// Additional fields that may be present at runtime but
// are ignored by validation or parsing
type QuestionRuntime = {
  // ID used internally to make items unique
  id: string;
  // Shadow marker state for drag dropping
  [SHADOW_ITEM_MARKER_PROPERTY_NAME]?: undefined | boolean;
  // Additional runtime image data
  image: {
    // Preloaded image in cases where images are used
    preloaded?: HTMLImageElement;
  } | null;
};

// Question type inferred from its schema
export type Question = z.infer<typeof questionSchema> & QuestionRuntime;
