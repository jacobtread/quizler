import type { GameConfig, GameState, RemoveReason } from "./game";
import type { Question } from "./question";
import type { Score, Scores } from "./score";
import type { SessionId } from "./session";

// Server message types
export const enum ServerEvent {
  PlayerData = "PlayerData",
  GameState = "GameState",
  Timer = "Timer",
  Question = "Question",
  Scores = "Scores",
  Score = "Score",
  Kicked = "Kicked",
  ResumptionToken = "ResumptionToken",
  ResumedGame = "ResumedGame"
}

// Server message schema based on each message type
export type ServerEventSchema = { ret: undefined } & (
  | { ty: ServerEvent.PlayerData; id: SessionId; name: string }
  | { ty: ServerEvent.GameState; state: GameState }
  | {
      ty: ServerEvent.Timer;
      value: number;
    }
  | { ty: ServerEvent.Question; question: Question }
  | { ty: ServerEvent.Scores; scores: Scores }
  | { ty: ServerEvent.Score; score: Score }
  | { ty: ServerEvent.Kicked; id: SessionId; reason: RemoveReason }
  | { ty: ServerEvent.ResumptionToken; token: string }
  | {
      ty: ServerEvent.ResumedGame;
      id: SessionId;
      host: boolean;
      name: string | null;
      token: string;
      config: GameConfig;
    }
);

// Server message type extractor
export type ServerEventOf<T> = Extract<ServerEventSchema, { ty: T }>;
