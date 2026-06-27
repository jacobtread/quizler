<script lang="ts">
  import { getRandomMessage } from "$lib/utils/messages";
  import { slide } from "svelte/transition";

  const message = getRandomMessage("Answered");
</script>

<main class="main" transition:slide|global>
  <h1 class="title">Waiting...</h1>
  <p class="text">{message}</p>
</main>

<style>
  .main {
    width: 100%;
    height: 100%;
    display: flex;
    flex-flow: column;
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
</style>
