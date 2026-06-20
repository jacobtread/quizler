import { ClientMessage, HostAction } from "$api/models";
import type { SocketStore } from "$lib/stores/socket.svelte";
import type { GameData } from "$pages/Game.svelte";
import { confirmDialog } from "$stores/dialogStore";
import { type AppStateStore } from "$stores/state.svelte";

export async function doKick(socket: SocketStore, id: string) {
  try {
    await socket.send({
      ty: ClientMessage.Kick,
      id
    });
  } catch (e) {
    console.error("Error while attempting to kick", e);
  }
}

export async function doHostAction(
  socket: SocketStore,
  action: HostAction
): Promise<void> {
  try {
    await socket.send({
      ty: ClientMessage.HostAction,
      action
    });
  } catch (e) {
    console.error("Error while attempting host action", action, e);
  }
}

export async function leave(
  socket: SocketStore,
  state: AppStateStore,
  gameData: GameData
) {
  const message = gameData.host
    ? "Are you sure you want to leave? Leaving will remove all other players from the game"
    : "Are you sure you want to leave?";

  const result = await confirmDialog("Confirm Leave", message);

  if (!result) return;

  // Take back to the home screen
  state.setHome();

  // Kick self from game to leave
  doKick(socket, gameData.id);
}
