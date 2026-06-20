import type { SessionId } from "./session";

// Mapping between IDs and the scores
export type Scores = Record<SessionId, number>;

// Different score types
export const enum ScoreType {
  Correct = "Correct",
  Incorrect = "Incorrect",
  Partial = "Partial"
}

// Score schemas for each different type
export type Score =
  | { ty: ScoreType.Correct; value: number }
  | {
      ty: ScoreType.Partial;
      count: number;
      total: number;
      value: number;
    }
  | { ty: ScoreType.Incorrect };
