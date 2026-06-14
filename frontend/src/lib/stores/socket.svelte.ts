import {
  ServerEvent,
  type ServerEventOf,
  ServerError,
  ServerResponse,
  ClientMessage,
  type ClientMessageOf,
  type ServerResponseOf,
  type ServerMessage,
  isServerEventType
} from "$api/models";
import { getServerURL } from "$api/http";
import type { AppStateStore } from "./state.svelte";

// Handler function that expects a specific server message type
type ServerEventHandler<Type> = (msg: ServerEventOf<Type>) => void;
// Handler function that expects a specific server message type
type ServerResponseHandler<Type> = (msg: ServerResponseOf<Type>) => void;

// Type of a handler for handling a message response
interface RequestHandler<T> {
  // Handler resolve callback with the response
  resolve: ServerResponseHandler<T>;
  // Handler reject callback with a server error
  reject(err: ServerError): void;
}

export type HandlerDestructor = VoidFunction;

export interface SocketStore {
  /** Socket readiness state */
  ready: boolean;

  /**
   * Subscribes to a specific type of message within a svelte
   * component context.
   *
   * Handles removing the handler on the component destroy
   *
   * @param ty The server event message type
   * @param handler The handler for the message
   * @param abort Optional abort signal to invoke the destructor
   * @return Destructor function to remove the handler
   */
  setHandler<T extends ServerEvent>(
    ty: T,
    handler: ServerEventHandler<T>,
    abort?: AbortSignal
  ): HandlerDestructor;

  /**
   * Sends the provided message to the server through
   * the socket
   *
   * @throws The server error or socket error
   * @param msg The message to send
   * @return The promise to the message response
   */
  send<T extends ClientMessage>(
    msg: ClientMessageOf<T>
  ): Promise<ServerResponseOf<T>>;
}

interface TypedHandlerWithAbort<K extends ServerEvent> {
  handler: ServerEventHandler<K>;
  abort?: AbortSignal;
}

export function createSocketState(appState: AppStateStore): SocketStore {
  let ready = $state(false);

  // Token for resuming the connection
  let resumptionToken: string | null =
    sessionStorage.getItem("resumptionToken");

  // Handler for the next response
  let responseHandle: RequestHandler<unknown> | null = null;
  // Queue of messages that haven't yet been handled
  let messageQueue: ServerMessage[] = [];

  // Currently set message handlers for handling messages
  const messageHandlers: Partial<
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    Record<ServerEvent, TypedHandlerWithAbort<any>>
  > = {};

  // Reference to the socket
  let socket: WebSocket = createSocket();

  /**
   * Logs out all the unhandled messages in the queue
   * then clears the queue
   */
  function clearMessageQueue() {
    for (const msg of messageQueue) {
      console.warn("Message was not handled", msg);
    }

    messageQueue = [];
  }

  /**
   * Creates a new socket connection assigning
   * the event handlers for the different events
   */
  function createSocket(): WebSocket {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- This is not reactive code it doesn't need it
    const params = new URLSearchParams();

    if (resumptionToken !== null) {
      params.set("resume", resumptionToken);
    }

    const socketParams = params.size > 0 ? `?${params.toString()}` : "";
    const socketPath = `/api/quiz/socket${socketParams}`;

    const socketUrl = getServerURL(socketPath);
    // Replace the url protocol with the correct socket protocol
    socketUrl.protocol = socketUrl.protocol.includes("https") ? "wss" : "ws";

    console.debug("Connecting to socket server " + socketUrl);

    // Create the socket
    const ws = new WebSocket(socketUrl);

    // Assign the message handler
    ws.onmessage = onMessage;

    // Handle open events
    ws.onopen = () => {
      // Handle the WebSocket connection becoming OPEN
      if (ws.readyState == WebSocket.OPEN) {
        console.debug("Connected to socket");
        ready = true;
      }
    };

    // Handle close events
    ws.onclose = (event: CloseEvent) => {
      // Handle the socket becoming unavailable
      console.error("WebSocket connection closed", event);

      // Update lost connection states
      onDisconnected();
    };

    // Handle error events
    ws.onerror = (event: Event) => {
      // Handle the socket becoming unavailable
      console.error("WebSocket error", event);
    };

    return ws;
  }

  function onDisconnected() {
    // Reset the state
    responseHandle = null;
    clearMessageQueue();

    // Return to the home screen
    appState.setHome();
    // Attempt to reconnect
    queueReconnect();
  }

  /**
   * Handles clearing the socket ready state and setting
   * a timeout for when to reconnect
   */
  function queueReconnect() {
    console.debug("Socket connection lost (Reconnecting in 1000ms)");

    ready = false;

    // Don't immediately try to reconnect
    setTimeout(() => {
      // Try reconnect the socket
      socket = createSocket();
    }, 1000);
  }

  function send<T extends ClientMessage>(
    msg: ClientMessageOf<T>
  ): Promise<ServerResponseOf<T>> {
    return new Promise((resolve, reject) => {
      console.debug("Sending message to server", msg);

      responseHandle = {
        resolve: resolve as ServerResponseHandler<unknown>,
        reject
      };

      const data = JSON.stringify(msg);
      try {
        socket.send(data);
      } catch (e) {
        console.error("Failed to send message", e);
        responseHandle = null;
        reject({
          ty: ServerResponse.Error,
          error: "Unexpected"
        });
      }
    });
  }

  /**
   * Event handler for parsing, and handling messages
   * received from the server
   *
   * @param data The message data from the event
   */
  function onMessage({ data }: MessageEvent) {
    // Parse the message
    const msg: ServerMessage = JSON.parse(data);

    // Ensure the message type is specified
    if (msg.ty === undefined) {
      console.error("Packet missing message type", data);
      return;
    }

    // Handle messages marked as responses
    if (msg.ret === 1) {
      if (responseHandle === null) {
        console.error("Missing response handle for message", msg);
        return;
      }

      // Compare the message type
      if (msg.ty === ServerResponse.Error) {
        // If the types didn't match it must've been an error
        responseHandle.reject(msg.error);
      } else {
        // Resolve with the correct result
        responseHandle.resolve(msg);
      }

      return;
    }

    // Find the handler for the message
    const entry = messageHandlers[msg.ty];
    if (entry !== undefined && (!entry.abort || !entry.abort.aborted)) {
      const handler = entry.handler as ServerEventHandler<unknown>;
      // Call the handler with the message
      handler(msg);
      return;
    }

    // Push the message to the queue
    messageQueue.push(msg);
  }

  function setHandler<T extends ServerEvent>(
    ty: T,
    handler: ServerEventHandler<T>,
    abort?: AbortSignal
  ) {
    messageHandlers[ty] = { handler, abort };
    console.debug("Added handler for", ty);

    // Process matching queued messages
    messageQueue = messageQueue.filter((msg) => {
      if (isServerEventType(ty, msg) && (!abort || !abort.aborted)) {
        // Handle messages that match the handler type
        handler(msg);
        return false;
      } else {
        return true;
      }
    });

    let destroyed = false;

    const destructor = () => {
      if (destroyed) return;
      delete messageHandlers[ty];
      destroyed = true;
    };

    abort?.addEventListener("abort", destructor);

    return destructor;
  }

  setHandler(ServerEvent.ResumptionToken, (event) => {
    resumptionToken = event.token;
    sessionStorage.setItem("resumptionToken", resumptionToken);
  });

  return {
    get ready() {
      return ready;
    },
    setHandler,
    send
  };
}
