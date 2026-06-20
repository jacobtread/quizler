import type z from "zod";
import type { createDataSchema } from "$api/schema/game";
import type { Question } from "./question";
import type { SessionId } from "./session";

export type CreateData = z.infer<typeof createDataSchema>;

export type CreateDataRuntime = {
  // Replace questions typing to include the runtime data
  questions: Question[];
} & CreateData;

// Quiz configuration
export interface GameConfig {
  // Name of the quiz
  name: string;
  // Text description of the quiz
  text: string;
}

// Represents a 5 character game token (e.g EAU32)
export type GameToken = string;

// Possible game states
export const enum GameState {
  Lobby = "Lobby",
  Starting = "Starting",
  AwaitingReady = "AwaitingReady",
  PreQuestion = "PreQuestion",
  AwaitingAnswers = "AwaitingAnswers",
  Marked = "Marked",
  Finished = "Finished"
}

// Different remove reasons
export const enum RemoveReason {
  RemovedByHost = "RemovedByHost",
  HostDisconnect = "HostDisconnect",
  LostConnection = "LostConnection",
  Disconnected = "Disconnected"
}

// Messages for different removal reasons
export const removeReasonText: Record<RemoveReason, string> = {
  [RemoveReason.RemovedByHost]: "Removed by host",
  [RemoveReason.HostDisconnect]: "Quiz host left",
  [RemoveReason.LostConnection]: "Connection lost",
  [RemoveReason.Disconnected]: "Disconnected"
};

// Snapshot of the game state at completion
// to keep around the scores and players
export interface GameSummary {
  /// Summary for each of the players in the game
  players: PlayerSummary[];
}

// Extended player data to include score
export type PlayerSummary = PlayerData & { score: number };

// Response structure for a created quiz
export interface CreatedResponse {
  // UUID of the prepared game
  uuid: string;
}

// Basic player data
export interface PlayerData {
  // The ID of the player
  id: SessionId;
  // The name of the player
  name: string;
}

// Actions that hosts can send to the server
export const enum HostAction {
  Start = "Start",
  Next = "Next",
  Reset = "Reset"
}
