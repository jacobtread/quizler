<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { doHostAction } from "$api/actions";
  import { GameState, HostAction } from "$api/models";

  import type { GameData } from "$pages/Game.svelte";
  import { formatTime } from "$lib/utils/utils";
  import socketContext from "$lib/context/socket";
  import type { TimerStore } from "$lib/stores/timerStore.svelte";

  interface Props {
    gameState: GameState;
    gameData: GameData;
    timerStore: TimerStore;
  }

  const { gameState, timerStore, gameData }: Props = $props();

  const socket = socketContext.get();
  const timeMs = $derived(timerStore.current);

  // Sends the next state action
  const next = () => doHostAction(socket, HostAction.Next);

  // Sends the host cancel action
  const reset = () => doHostAction(socket, HostAction.Reset);
</script>

<main class="page page--overflow" transition:slide|global>
  <div class="quiz">
    {#if timeMs > 1000}
      <span class="time" transition:slide>{formatTime(timeMs)}</span>
    {/if}

    {#if gameState === GameState.Starting}
      <p class="text">Get ready, the quiz is about to start.</p>
    {:else}
      <p class="text">Get ready to answer</p>
    {/if}
  </div>

  <div class="bottom">
    <p class="token">{gameData.token}</p>

    {#if gameData.host}
      {#if gameState === GameState.Starting}
        <button class="btn btn--surface" onclick={reset}>Cancel</button>
      {/if}
      <button class="btn btn--surface" onclick={next}>Skip</button>
    {/if}
  </div>
</main>

<style>
  .quiz {
    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;
    flex: auto;
    text-align: center;

    padding: 1rem;
    background: linear-gradient(
      to bottom right,
      var(--starting-start),
      var(--starting-end)
    );
  }

  .time {
    font-size: 5rem;

    color: #fff;
    text-shadow: 0 3px 1px var(--starting-start-shadow);
  }

  .text {
    font-size: 1.25rem;

    margin-bottom: 1rem;
    color: var(--starting-start-shadow-dark);
  }

  .bottom {
    width: 100%;
    background-color: var(--surface);
    padding: 0.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 5px solid var(--surface-light);
    padding-left: 1rem;
    gap: 1rem;
  }

  .token {
    color: #fff;
    font-weight: bold;
    font-size: 1.5rem;
  }
</style>
