<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { onMount } from "svelte";
  import QRCode from "qrcode";

  import { doHostAction, doKick, leave } from "$api/actions";
  import {
    type PlayerData,
    type SessionId,
    GameState,
    HostAction
  } from "$api/models";

  import ScoreTweened from "$components/TweenedValue.svelte";
  import type { GameData } from "$pages/Game.svelte";
  import stateContext from "$lib/context/state";
  import socketContext from "$lib/context/socket";

  interface Props {
    gameData: GameData;
    players: PlayerData[];
    scores: Record<SessionId, number>;
    gameState: GameState;
  }

  const { gameData, players, scores, gameState }: Props = $props();

  const appState = stateContext.get();
  const socketState = socketContext.get();

  let qrCanvas: HTMLCanvasElement;
  let qrError = $state(false);

  // Build a player invite link using the same token shown in the lobby.
  const joinUrl = $derived.by(() => {
    const url = new URL(window.location.href);
    url.search = "";
    url.hash = "";
    url.searchParams.set("token", gameData.token);
    return url.toString();
  });

  onMount(() => {
    void QRCode.toCanvas(qrCanvas, joinUrl, { width: 220, margin: 1 }).catch(
      (error: unknown) => {
        console.error("Could not generate the invite QR code", error);
        qrError = true;
      }
    );
  });

  // Sends the next state action
  const next = () => doHostAction(socketState, HostAction.Next);
</script>

<main class="page page--middle page--overflow" transition:slide|global>
  <div class="quiz">
    <h1 class="token">
      {gameData.token}
    </h1>

    <h2 class="name">{gameData.config.name}</h2>
    <p class="desc">{gameData.config.text}</p>

    <section class="invite" aria-label="Join this quiz">
      <div class="invite__qr-wrap">
        <canvas
          class="invite__qr"
          bind:this={qrCanvas}
          role="img"
          aria-label={`QR code to join ${gameData.config.name}`}
          hidden={qrError}
        ></canvas>
        {#if qrError}
          <p>QR code unavailable. Use the link to join.</p>
        {/if}
      </div>
      <div>
        <h3>Scan to join</h3>
        <p>Scan this code to open the quiz join page.</p>
        <a class="invite__link" href={joinUrl}>{joinUrl}</a>
      </div>
    </section>

    <div class="btn-row btn-row--fill actions">
      <button
        class="btn"
        onclick={() => leave(socketState, appState, gameData)}
      >
        Leave
      </button>

      {#if gameState === GameState.Marked}
        <!-- Cancel started button for starting games -->
        <button class="btn" onclick={next}>Next</button>
      {:else if players.length > 0 && gameState === GameState.Lobby}
        <!-- Start button if theres players in the game -->
        <button class="btn" onclick={next}>Start</button>
      {/if}
    </div>

    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Score</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each players as player (player.id)}
          <tr class="player" animate:flip>
            <td class="player__name">
              {player.name}
            </td>
            <td class="player_score">
              <ScoreTweened value={scores[player.id] ?? 0} />
            </td>
            <!-- Host privileges -->
            <td class="player__action">
              <button
                class="btn"
                onclick={() => doKick(socketState, player.id)}
              >
                Kick
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</main>

<style>
  .page {
    padding: 1rem;
  }

  .quiz {
    width: 100%;
    max-width: 48rem;
  }

  .actions {
    margin-bottom: 1rem;
  }

  .invite {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 1rem 0;
    padding: 1rem;
    border-radius: 0.5rem;
    background-color: var(--surface);
  }

  .invite__qr-wrap {
    flex: 0 0 11rem;
  }

  .invite__qr {
    display: block;
    width: 11rem;
    height: 11rem;
    background: white;
  }

  .invite__link {
    overflow-wrap: anywhere;
  }

  .player {
    position: relative;
    background-color: var(--surface);
    border: 1px solid var(--surface-light);
  }

  .player__name {
    width: 100%;
    padding: 1rem;
  }

  .player__action {
    padding: 0.5rem;
    display: flex;
    flex-flow: row;
    justify-content: center;
    border: none;
  }

  .token {
    color: var(--primary);
    font-weight: bold;
    font-size: 3rem;
    margin: 0;
    width: 100%;
  }

  .desc {
    margin-bottom: 1rem;
  }

  @media screen and (max-width: 32rem) {
    .invite {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
