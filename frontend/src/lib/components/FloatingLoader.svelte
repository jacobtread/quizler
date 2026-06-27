<script lang="ts">
  import { fade } from "svelte/transition";
  import Spinner from "$components/Spinner.svelte";

  interface Props {
    text?: string;
  }

  let { text = "" }: Props = $props();
</script>

<main class="main" transition:fade>
  <Spinner />
  <p class="text">{text}</p>
</main>

<style>
  .text {
    margin-top: 1rem;
  }

  .main {
    z-index: 2;
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;

    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;

    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }
</style>
