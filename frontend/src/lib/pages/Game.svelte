<script module lang="ts">
  import type { GameConfig, PlayerSummary, Scores } from "$api/models";

  export interface GameData {
    // ID of the current player
    id: SessionId;
    // Current game token
    token: string;
    // Current game config
    config: GameConfig;
    // Whether we are the host
    host: boolean;
    // The current player name
    name?: string | undefined;
  }
</script>

<script lang="ts">
  import {
    ServerEvent,
    type PlayerData,
    GameState,
    type Question,
    type Score,
    type SessionId,
    ScoreType,
    removeReasonText,
    type GameSummary,
    RemoveReason,
    ClientMessage
  } from "$api/models";

  import { errorDialog } from "$stores/dialogStore";

  import AnsweredView from "$pages/game/AnsweredView.svelte";
  import FinishedView from "$pages/game/FinishedView.svelte";
  import QuestionView from "$pages/game/QuestionView.svelte";
  import LobbyView from "$pages/game/LobbyView.svelte";
  import ScoreView from "$pages/game/ScoreView.svelte";
  import Waiting from "$pages/game/Waiting.svelte";
  import Starting from "$pages/game/Starting.svelte";
  import Loading from "$pages/Loading.svelte";
  import stateContext from "$lib/context/state";
  import socketContext from "$lib/context/socket";
  import { onMount } from "svelte";
  import { createTimerStore } from "$lib/stores/timerStore.svelte";
  import { preloadImage } from "$lib/api/http";

  interface Props {
    gameData: GameData;
  }

  let { gameData }: Props = $props();

  const appState = stateContext.get();
  const socket = socketContext.get();

  const timerStore = createTimerStore();

  // Player data loaded over the network
  let remotePlayerData: PlayerData[] = $state([]);

  let gameState: GameState = $state(GameState.Lobby);

  // The current game summary
  let summary: GameSummary | null = $state(null);

  let question: Question | null = $state(null);

  let score: Score = $state({ ty: ScoreType.Incorrect });
  let scores: Scores = $state({});

  let answered: boolean = $state(false);

  // Fallback player list to use before remote players are loaded
  const defaultPlayers = $derived(
    gameData.host ? [] : [{ id: gameData.id, name: gameData.name ?? "" }]
  );

  // Current player list ordered by scores
  const players: PlayerSummary[] = $derived.by(() => {
    const players =
      remotePlayerData.length > 0 ? remotePlayerData : defaultPlayers;

    // Add the scoring data to the player
    const playersWithScore = players.map((player) => ({
      score: scores[player.id] ?? 0,
      ...player
    }));

    // Sort players list by the player scores if available
    playersWithScore.sort((a, b) => b.score - a.score);

    return playersWithScore;
  });

  async function preloadQuestionImage() {
    if (!question) return;
    // Preload the image
    const preloadedImage = await preloadImage(gameData.token, question);

    if (preloadedImage !== null && question.image !== null) {
      // Ensure browser compatibility
      if (preloadedImage.decode !== undefined) {
        await preloadedImage.decode();
      }

      question.image.preloaded = preloadedImage;
    }
  }

  async function markReady() {
    // Update the ready state
    try {
      await socket.send({ ty: ClientMessage.Ready });
      console.debug("Server acknowledged ready state");
    } catch (e) {
      console.error("Error while attempting to ready", e);
    }
  }

  onMount(() => {
    const abortController = new AbortController();
    const abortSignal = abortController.signal;

    // Hook the handlers for the different message types
    socket.setHandler(
      ServerEvent.PlayerData,
      (msg) => {
        console.debug("Other player message", msg);

        // Add to the players list
        remotePlayerData.push(msg);
      },
      abortSignal
    );

    socket.setHandler(
      ServerEvent.GameState,
      (msg) => {
        console.debug("Game state message", msg);
        gameState = msg.state;

        // If the state has changed reset our answered state
        answered = false;

        // Reset known scores when reverting to lobby state
        if (msg.state === GameState.Lobby) {
          scores = {};
        } else if (msg.state === GameState.Finished) {
          summary = { players };
        }
      },
      abortSignal
    );

    socket.setHandler(
      ServerEvent.Timer,
      (msg) => {
        console.debug("Time sync message", msg);
        timerStore.start(msg.value);
      },
      abortSignal
    );

    socket.setHandler(
      ServerEvent.Question,
      async (msg) => {
        console.debug("Question message", msg);
        question = msg.question;

        await preloadQuestionImage();
        await markReady();
      },
      abortSignal
    );

    socket.setHandler(
      ServerEvent.Scores,
      (msg) => {
        console.debug("Score message", msg);
        scores = msg.scores;
      },
      abortSignal
    );

    socket.setHandler(
      ServerEvent.Score,
      (msg) => {
        console.debug("Score message", msg);
        score = msg.score;
      },
      abortSignal
    );

    socket.setHandler(
      ServerEvent.Kicked,
      (msg) => {
        console.debug("Kick message", msg);
        // Remove from the players list
        remotePlayerData = remotePlayerData.filter(
          (player) => player.id !== msg.id
        );

        // if the removed player was us
        if (msg.id === gameData.id) {
          appState.setHome();

          // For remove reasons other than self disconnect
          if (msg.reason !== RemoveReason.Disconnected) {
            const reason = removeReasonText[msg.reason];
            errorDialog("Removed from game", reason);
          }
        }
      },
      abortSignal
    );

    return () => {
      abortController.abort();
      timerStore.reset();
    };
  });
</script>

{#if gameState === GameState.Finished && summary != null}
  <FinishedView {gameData} {summary} />
{:else if gameState === GameState.Starting || gameState === GameState.PreQuestion || gameState === GameState.AwaitingReady}
  <Starting {gameState} {gameData} {timerStore} />
{:else if gameState === GameState.AwaitingAnswers && question != null}
  {#if !answered}
    <QuestionView {gameData} {question} {timerStore} bind:answered />
  {:else if players.length !== 1}
    <!--
      Don't bother showing answered screen if only one player
      as it will just be a blink before the score screen
    -->
    <AnsweredView />
  {/if}
{:else if gameData.host}
  <LobbyView {gameData} {gameState} {players} {scores} />
{:else if gameState === GameState.Lobby}
  <Waiting {gameData} />
{:else if gameState === GameState.Marked}
  <ScoreView {score} />
{:else}
  <!-- Just dot for the message while waiting for a state -->
  <Loading text="..." />
{/if}
