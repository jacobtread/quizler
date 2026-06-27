<script lang="ts">
  import { slide } from "svelte/transition";

  import GitHub from "$components/icons/GitHub.svelte";
  import Play from "$components/icons/Play.svelte";
  import Edit from "$components/icons/Edit.svelte";
  import Logo from "$components/icons/Logo.svelte";
  import stateContext from "$lib/context/state";

  const appState = stateContext.get();
</script>

<main class="main" transition:slide|global>
  <div class="left">
    <div class="logo">
      <Logo />
    </div>

    <a
      href="https://github.com/jacobtread/Quizler"
      target="_blank"
      rel="noreferrer"
      title="View on Github"
      class="btn btn--icon github"
    >
      <GitHub />
      View on GitHub
    </a>
  </div>
  <div class="actions">
    <button onclick={appState.setConnect} class="action" aria-label="Join">
      <Play />
      <div class="action__body">
        <p class="action__name">Join a quiz</p>
        <p class="action__text">Enter a game code and hop right in</p>
      </div>
    </button>

    <button onclick={appState.setCreate} class="action" aria-label="Create">
      <Edit />
      <div class="action__body">
        <p class="action__name">Create a quiz</p>
        <p class="action__text">Create your own quiz</p>
      </div>
    </button>
  </div>
</main>

<style>
  .left {
    text-align: center;
  }

  .logo {
    margin: 1rem auto;
  }

  .github {
    display: inline-block;
    padding: 0.75rem;
    border-radius: 0.5rem;
  }

  .main {
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: auto;
    background-color: var(--app-background);
    background-image: var(--app-background-pattern);

    gap: 3rem;
    height: 100%;
  }

  .logo :global(> svg) {
    max-width: 16rem;
    padding: 1rem;
    fill: var(--logo-fill);
  }

  .action {
    position: relative;
    overflow: hidden;

    width: 100%;

    align-items: center;
    gap: 1rem;

    padding: 1rem;
    margin-bottom: 1rem;

    border-radius: 1rem;

    background-color: var(--surface);
    border: 0.15rem solid var(--btn-border-color);
    text-align: left;

    z-index: 2;

    cursor: pointer;

    transition:
      background-color 0.5s ease,
      color 0.2s linear;
  }

  .action :global(> svg) {
    float: left;
    margin-right: 1rem;
    padding: 1rem;
    box-sizing: content-box;
    background-color: var(--surface-light);
    border-radius: 0.5rem;
    transition:
      background-color 0.5s ease,
      color 0.2s linear;
  }

  .action__name {
    font-size: 1.4rem;
    margin-bottom: 0.25rem;
    color: var(--text-primary);
  }

  .action__text {
    color: var(--text-secondary);
    font-size: 1.1rem;
  }

  .action:before {
    content: "";

    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    z-index: -1;

    background: var(--primary-dark);
    border-radius: 1rem;

    transform: translate(-100%);

    transition: 0.5s ease;
  }

  .action:hover {
    border-color: var(--primary);
  }

  .action:hover:before {
    transform: translate(0);
  }

  .action:hover :global(> svg) {
    background-color: var(--primary);
  }

  @media screen and (max-height: 48rem) and (max-width: 52rem) {
    .main {
      justify-content: start;
      align-items: center;
      padding: 1rem 0;
      gap: 1rem;
    }
  }

  @media screen and (max-width: 52rem) {
    .main {
      flex-flow: column;
      gap: 1rem;
    }
  }

  @media screen and (max-width: 32rem) {
    .actions {
      margin: 1rem;
    }

    .action :global(> svg) {
      display: block;
      float: none;
      width: calc(100% - 2rem);
      max-height: 3rem;
      margin-bottom: 1rem;
    }

    .action__body {
      display: block;
      width: 100%;
    }
  }

  @media screen and (max-width: 16rem) {
    .logo {
      width: 100%;
      overflow: hidden;
      padding: 0 1rem;
      margin: 0 auto;
    }

    .logo :global(> svg) {
      width: 100%;
    }

    .actions {
      text-align: center;
      margin: 0;
    }

    .action {
      width: auto;
    }

    .action__body {
      display: none;
    }

    .action :global(> svg) {
      padding: 1rem;
      width: auto;
      margin-bottom: 0;
      margin-right: 0;
    }

    .github {
      display: none;
    }
  }

  @media screen and (max-height: 48rem) {
    .github {
      display: none;
    }
  }

  @media screen and (max-height: 42rem) {
    .logo :global(> svg) {
      max-height: 6rem;
      max-width: none;
      padding: 1rem;
    }

    .logo {
      margin: 0 auto;
    }

    .main {
      gap: 0;
    }
  }

  @media screen and (max-height: 35rem) {
    .logo {
      display: none;
    }
  }
</style>
