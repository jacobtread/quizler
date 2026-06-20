import type {
  ServerEvent,
  ServerEventOf,
  ServerEventSchema
} from "./serverEvent";
import type { ServerResponseSchema } from "./serverResponse";

export * from "./answer";
export * from "./game";
export * from "./nameFiltering";
export * from "./question";
export * from "./score";
export * from "./serverError";
export * from "./session";
export * from "./clientMessage";
export * from "./serverEvent";
export * from "./serverResponse";
export * from "./image";

export type ServerMessage = ServerResponseSchema | ServerEventSchema;

export function isServerEventType<T extends ServerEvent>(
  ty: T,
  msg: ServerMessage
): msg is ServerEventOf<T> {
  return msg.ty === ty;
}
