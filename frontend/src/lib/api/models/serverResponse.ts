import type { SessionId } from "./session";
import type { GameConfig } from "./game";
import type { ServerError } from "./serverError";
import type { ClientMessage } from "./clientMessage";
import type { ServerEventOf } from "./serverEvent";

// Server response message type
export const enum ServerResponse {
  Joined = "Joined",
  Ok = "Ok",
  Error = "Error"
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
