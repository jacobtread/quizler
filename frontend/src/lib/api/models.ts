import {
  MAX_ANSWERS,
  MAX_ANSWER_LENGTH,
  MAX_DESCRIPTION_LENGTH,
  MAX_MAX_PLAYERS,
  MAX_QUESTIONS,
  MAX_QUESTION_LENGTH,
  MAX_TITLE_LENGTH,
  MIN_MAX_PLAYERS
} from "$lib/constants";
import type { SHADOW_ITEM_MARKER_PROPERTY_NAME } from "svelte-dnd-action";
import { z } from "zod";

// Represents a unique ID of a session
export type SessionId = string;

// Represents a 5 character game token (e.g EAU32)
export type GameToken = string;

// Mapping between IDs and the scores
export type Scores = Record<SessionId, number>;

// UUID to an image on the server
export type ImageRef = string;

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

// Quiz configuration
export interface GameConfig {
  // Name of the quiz
  name: string;
  // Text description of the quiz
  text: string;
}

// Server error types
export const enum ServerError {
  MalformedMessage = "MalformedMessage",
  InvalidToken = "InvalidToken",
  InvalidNameLength = "InvalidNameLength",
  UsernameTaken = "UsernameTaken",
  InappropriateName = "InappropriateName",
  NotJoinable = "NotJoinable",
  CapacityReached = "CapacityReached",
  UnknownPlayer = "UnknownPlayer",
  Unexpected = "Unexpected",
  InvalidPermission = "InvalidPermission",
  UnexpectedMessage = "UnexpectedMessage",
  InvalidAnswer = "InvalidAnswer"
}

// Messages for different server errors
export const errorText: Record<ServerError, string> = {
  [ServerError.MalformedMessage]: "Unknown client sent invalid message",
  [ServerError.InvalidToken]: "Invalid token provided",
  [ServerError.InvalidNameLength]: "Invalid name length",
  [ServerError.UsernameTaken]: "Username already in use",
  [ServerError.InappropriateName]:
    "That name is not allowed/inappropriate choose another name",
  [ServerError.NotJoinable]: "Quiz is not joinable",
  [ServerError.CapacityReached]: "Quiz is full",
  [ServerError.UnknownPlayer]: "Target player not found",
  [ServerError.Unexpected]: "Unexpected error occurred",
  [ServerError.InvalidPermission]: "You don't have permission to do that",
  [ServerError.UnexpectedMessage]: "Client and server out of sync",
  [ServerError.InvalidAnswer]: "Invalid answer type"
};

// Name filtering modes
export enum NameFiltering {
  // Don't filter names anything goes
  None = "None",
  // Only stop the more severe names
  Low = "Low",
  // Stop anything thats above mild
  Medium = "Medium",
  // Filter out any names that might be inappropriate
  High = "High"
}

export const nameFilterText: Record<NameFiltering, string> = {
  [NameFiltering.None]: "Don't filter names",
  [NameFiltering.Low]: "Filter out more severe names",
  [NameFiltering.Medium]: "Filter out anything thats not mild",
  [NameFiltering.High]: "Filter out as much as possible"
};

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

// Actions that hosts can send to the server
export const enum HostAction {
  Start = "Start",
  Next = "Next",
  Reset = "Reset"
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

// Question types
export enum QuestionType {
  Single = "Single",
  Multiple = "Multiple",
  TrueFalse = "TrueFalse",
  Typer = "Typer"
}

// Piece of text representing an answer
const answerText = z
  .string()
  .trim()
  .nonempty("cannot be empty")
  .max(MAX_ANSWER_LENGTH, `cannot be longer than ${MAX_ANSWER_LENGTH}`);

// Schema for question answers
const answerValueSchema = z.object({
  id: z.number(),
  value: answerText,
  correct: z.boolean()
});

// Answer value type inferred from its schema
export type AnswerValue = z.infer<typeof answerValueSchema>;

export enum ImageFit {
  Contain = "Contain",
  Cover = "Cover",
  Width = "Width",
  Height = "Height"
}

export const imageFitText: Record<ImageFit, string> = {
  [ImageFit.Contain]: "Fit the entire image",
  [ImageFit.Cover]: "Fill the available space",
  [ImageFit.Width]: "Fill available width",
  [ImageFit.Height]: "Fill available height"
};

// Schema for questions
export const questionSchema = z
  .object({
    text: z
      .string()
      .trim()
      .nonempty("Question cannot be empty")
      .max(MAX_QUESTION_LENGTH),
    image: z
      .object({
        uuid: z.string().uuid(),
        fit: z.nativeEnum(ImageFit)
      })
      .nullable(),
    answer_time: z.number(),
    bonus_score_time: z.number(),
    scoring: z.object({
      min_score: z.number(),
      max_score: z.number(),
      bonus_score: z.number()
    })
  })
  .and(
    z.discriminatedUnion("ty", [
      // Single choice questions
      z.object({
        ty: z.literal(QuestionType.Single),
        answers: z
          .array(answerValueSchema)
          .min(1, "Must provide at least one answer")
          .max(
            MAX_ANSWERS,
            `Too many answers maximum allowed is ${MAX_ANSWERS}`
          )
      }),
      // Multiple choice questions
      z.object({
        ty: z.literal(QuestionType.Multiple),
        answers: z
          .array(answerValueSchema)
          .min(1, "Must provide at least one answer")
          .max(
            MAX_ANSWERS,
            `Too many answers maximum allowed is ${MAX_ANSWERS}`
          ),
        correct_answers: z.number()
      }),
      // True / False choice questions
      z.object({
        ty: z.literal(QuestionType.TrueFalse),
        answer: z.boolean()
      }),

      // Typing question
      z.object({
        ty: z.literal(QuestionType.Typer),
        answers: z
          .array(answerText)
          .min(1, "Must provide at least one answer")
          .max(
            MAX_ANSWERS,
            `Too many answers maximum allowed is ${MAX_ANSWERS}`
          ),
        ignore_case: z.boolean()
      })
    ])
  );

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

export const createDataSchema = z.object({
  name: z.string().trim().max(MAX_TITLE_LENGTH),
  text: z.string().trim().max(MAX_DESCRIPTION_LENGTH),
  max_players: z.number().min(MIN_MAX_PLAYERS).max(MAX_MAX_PLAYERS),
  filtering: z.nativeEnum(NameFiltering),
  questions: z.array(questionSchema).min(1).max(MAX_QUESTIONS)
});

export type CreateData = z.infer<typeof createDataSchema>;

export type CreateDataRuntime = {
  // Replace questions typing to include the runtime data
  questions: Question[];
} & CreateData;

// Client message types
export const enum ClientMessage {
  Initialize = "Initialize",
  Connect = "Connect",
  Join = "Join",
  Ready = "Ready",
  HostAction = "HostAction",
  Answer = "Answer",
  Kick = "Kick"
}

// Client message schema based on each message type
export type ClientMessageSchema = {
  rid?: number;
} & (
  | { ty: ClientMessage.Initialize; uuid: string }
  | { ty: ClientMessage.Connect; token: GameToken }
  | { ty: ClientMessage.Join; name: string }
  | { ty: ClientMessage.Ready }
  | { ty: ClientMessage.HostAction; action: HostAction }
  | { ty: ClientMessage.Answer; answer: Answer }
  | { ty: ClientMessage.Kick; id: SessionId }
);

// Client message type extractor
export type ClientMessageOf<T> = Extract<ClientMessageSchema, { ty: T }>;

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

// Server response message type
export const enum ServerResponse {
  Joined = "Joined",
  Ok = "Ok",
  Error = "Error"
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

export function isServerEventType<T extends ServerEvent>(
  ty: T,
  msg: ServerMessage
): msg is ServerEventOf<T> {
  return msg.ty === ty;
}

export type ServerResponseSchema = { ret: 1 } & (
  | {
      ty: ServerResponse.Joined;
      id: SessionId;
      token: string;
      config: GameConfig;
    }
  | { ty: ServerResponse.Ok }
  | { ty: ServerResponse.Error; error: ServerError }
);

export type ServerMessage = ServerResponseSchema | ServerEventSchema;

// Mapping between client messages and the server message type
export type MessagePairs =
  | { left: ClientMessage.Initialize; right: ServerResponse.Joined }
  | { left: ClientMessage.Join; right: ServerResponse.Joined }
  | { left: ClientMessage.Connect; right: ServerResponse.Ok }
  | { left: ClientMessage.Ready; right: ServerResponse.Ok }
  | { left: ClientMessage.HostAction; right: ServerResponse.Ok }
  | { left: ClientMessage.Answer; right: ServerResponse.Ok }
  | { left: ClientMessage.Kick; right: ServerResponse.Ok };

// Server message type extractor
export type ServerResponseOf<T> = Extract<
  ServerResponseSchema,
  {
    ty: Extract<MessagePairs, { left: T }>["right"]; // Type is extracted by using the mapping to locate the right hand side
  }
>;

// Response message type from the client message
export type ResponseMessage<T> = ServerEventOf<ServerResponseOf<T>>;
