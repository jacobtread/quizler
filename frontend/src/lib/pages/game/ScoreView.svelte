<script lang="ts">
  import { slide } from "svelte/transition";
  import { Tween } from "svelte/motion";

  import { ScoreType, type Score } from "$api/models";
  import { getRandomMessage } from "$lib/utils/messages";

  interface Props {
    score: Score;
  }

  let { score }: Props = $props();

  const message: string = $derived(getRandomMessage(score.ty));

  const value = Tween.of(
    () => {
      if (score.ty === ScoreType.Correct || score.ty === ScoreType.Partial) {
        return score.value;
      }

      return 0;
    },
    {
      delay: 500
    }
  );
</script>

<main class="main" data-type={score.ty} transition:slide|global>
  <h1 class="title">{score.ty}</h1>
  <p class="text">{message}</p>
  {#if score.ty === ScoreType.Correct}
    <p class="score">+{value.current.toFixed(0)}</p>
  {:else if score.ty === ScoreType.Partial}
    <p class="ratio">{score.count} / {score.total}</p>
    <p class="score">+{value.current.toFixed(0)}</p>
  {/if}
</main>

<style>
  .text {
    color: #fff;
    text-shadow: 0 1px 2px #000;
    display: block;
    margin-bottom: 1rem;
  }

  .score {
    padding: 1rem;
    background-color: rgba(0, 0, 0, 0.3);
    border-radius: 0.5rem;
    color: #fff;
  }

  .main {
    width: 100%;
    height: 100%;
    display: flex;
    flex-flow: column;
    gap: 1rem;
    justify-content: center;
    align-items: center;
    background: linear-gradient(
      to bottom right,
      var(--primary),
      var(--secondary)
    );
  }

  .title {
    font-size: 3rem;
    color: #fff;
    text-shadow: 0 3px 1px var(--primary-shadow);
  }

  .text {
    font-size: 1.25rem;
    color: #fff;
    text-shadow: 0 3px 1px var(--primary-shadow);
  }

  .main[data-type="Correct"] {
    background: linear-gradient(
      to bottom right,
      var(--correct-start),
      var(--correct-end)
    );
  }

  .main[data-type="Correct"] .title {
    text-shadow: 0 3px 1px var(--correct-end-text-shadow);
  }

  .main[data-type="Correct"] .text {
    text-shadow: 0 2px 1px var(--correct-end-text-shadow);
  }

  .main[data-type="Partial"] {
    background: linear-gradient(
      to bottom right,
      var(--partial-start),
      var(--partial-end)
    );
  }

  .main[data-type="Partial"] .title {
    text-shadow: 0 3px 1px var(--partial-start-text-shadow);
  }

  .main[data-type="Partial"] .text {
    text-shadow: 0 2px 1px var(--partial-start-text-shadow);
  }

  .main[data-type="Incorrect"] {
    background: linear-gradient(
      to bottom right,
      var(--incorrect-start),
      var(--incorrect-end)
    );
  }

  .main[data-type="Incorrect"] .title {
    text-shadow: 0 3px 1px var(--incorrect-end-text-shadow);
  }

  .main[data-type="Incorrect"] .text {
    text-shadow: 0 2px 1px var(--incorrect-end-text-shadow);
  }
</style>
