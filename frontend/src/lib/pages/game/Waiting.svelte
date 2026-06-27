<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { leave } from "$api/actions";

  import type { GameData } from "$pages/Game.svelte";
  import stateContext from "$lib/context/state";
  import socketContext from "$lib/context/socket";

  interface Props {
    gameData: GameData;
  }

  const { gameData }: Props = $props();

  const appState = stateContext.get();
  const socket = socketContext.get();
</script>

<main class="page page--overflow" transition:slide|global>
  <div class="quiz">
    <h1 class="title">Waiting for host...</h1>
    <p class="text">You're in, waiting for the host to start the quiz.</p>
  </div>

  <div class="bottom">
    <p class="token">{gameData.token}</p>

    <button
      class="btn btn--surface"
      onclick={() => leave(socket, appState, gameData)}
    >
      Leave
    </button>
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
      var(--partial-start),
      var(--partial-end)
    );
  }

  .title {
    font-size: 3rem;

    color: #fff;
    text-shadow: 0 3px 1px var(--partial-start-text-shadow-dark);
  }

  .text {
    font-size: 1.25rem;

    margin-bottom: 1rem;
    color: var(--partial-start-text-shadow-darkest);
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
  }

  .token {
    color: #fff;
    font-weight: bold;
    font-size: 1.5rem;
  }

  @media screen and (max-width: 48rem) {
    .title {
      font-size: 2rem;
    }
  }
</style>
