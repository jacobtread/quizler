<script lang="ts">
  import { AppStateType, createAppState } from "$stores/state.svelte";

  import GlobalDialog from "$components/GlobalDialog.svelte";
  import Loading from "$pages/Loading.svelte";
  import Connect from "$pages/Connect.svelte";
  import Create from "$pages/Create.svelte";
  import Home from "$pages/Home.svelte";
  import Game from "$pages/Game.svelte";
  import stateContext from "$lib/context/state";
  import { createSocketState } from "$lib/stores/socket.svelte";
  import socketContext from "$lib/context/socket";
  import { onMount } from "svelte";
  import { ServerEvent } from "$lib/api/models";

  const state = createAppState();
  const socket = createSocketState(state);

  const appState = $derived(state.current);

  stateContext.set(state);
  socketContext.set(socket);

  onMount(() => {
    socket.recreate();

    const abortController = new AbortController();
    const abortSignal = abortController.signal;

    socket.setHandler(
      ServerEvent.ResumedGame,
      (msg) => {
        console.debug("Resumed Game", msg);

        const { id, host, token, config, name } = msg;
        state.setGame({ id, token, config, host, name: name ?? undefined });
      },
      abortSignal
    );

    return () => {
      abortController.abort();
      socket.cleanup();
    };
  });
</script>

{#if socket.ready}
  {#if appState.ty == AppStateType.Home}
    <Home />
  {:else if appState.ty === AppStateType.Create}
    <Create />
  {:else if appState.ty === AppStateType.Connect}
    <Connect />
  {:else if appState.ty === AppStateType.Game}
    {const gameData = appState.gameData}
    <!--
    ^ gameData must be captured as a local constant to avoid reactively passing the state which
    causes the gameData to become undefined when the socket handlers access it slightly before
    the component is unmounted when a game is left
    -->
    <Game {gameData} />
  {/if}
{:else}
  <Loading text="Connecting to server..." />
{/if}

<GlobalDialog />
