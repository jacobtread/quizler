import type { Answer } from "./answer";
import type { GameToken, HostAction } from "./game";
import type { SessionId } from "./session";

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
