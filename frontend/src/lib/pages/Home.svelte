<script lang="ts">
  import { slide } from "svelte/transition";

  import GitHub from "$assets/icons/github.svg?component";
  import Play from "$assets/icons/play.svg?component";
  import Edit from "$assets/icons/edit.svg?component";
  import Logo from "$assets/icons/logo.svg?component";
  import stateContext from "$lib/context/state";
  import HomeActionButton from "$lib/components/HomeActionButton.svelte";

  const appState = stateContext.get();
</script>

<main class="main" transition:slide|global>
  <div class="left">
    <div class="logo">
      <Logo style="width: 100%;height: 100%;display: block;" />
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
    <HomeActionButton
      icon={Play}
      label="Join a quiz"
      description="Enter a game code and hop right in"
      onClick={appState.setConnect}
      aria-label="Join"
    />

    <HomeActionButton
      icon={Edit}
      label="Create a quiz"
      description="Create your own quiz"
      onClick={appState.setCreate}
      aria-label="Create"
    />
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

  .logo {
    max-width: 16rem;
    padding: 1rem;
    color: var(--logo-fill);
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
  }

  @media screen and (max-width: 16rem) {
    .logo {
      width: 100%;
      overflow: hidden;
      padding: 0 1rem;
      margin: 0 auto;
    }

    .actions {
      text-align: center;
      margin: 0;
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
    .logo {
      height: 6rem;
      max-width: none;
      margin: 0 auto;
      padding: 1rem;
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
