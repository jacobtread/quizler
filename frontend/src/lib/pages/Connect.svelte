<script lang="ts">
  import { slide } from "svelte/transition";

  import {
    MAX_PLAYER_NAME_LENGTH,
    MIN_PLAYER_NAME_LENGTH,
    TOKEN_LENGTH
  } from "$lib/constants";

  import { ClientMessage, ServerError, errorText } from "$api/models";

  import FloatingLoader from "$components/FloatingLoader.svelte";
  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";

  import { errorDialog } from "$stores/dialogStore";
  import stateContext from "$lib/context/state";
  import socketContext from "$lib/context/socket";

  const State = {
    Connect: 0,
    Join: 1
  };

  const appState = stateContext.get();
  const socket = socketContext.get();

  let connectState = $state(State.Connect);

  // The user provided token
  let token: string = $state("");

  // The user provided name
  let name = $state("");

  // Disabled state for the connect button
  let tokenValid: boolean = $state(false);

  // Disabled state for the connect button
  let nameValid: boolean = $state(false);

  // Loading screen state
  let loading: boolean = $state(false);

  // Determines whether focus should be updated
  let updateFocus: boolean = $state(true);

  let inputToken: HTMLInputElement | undefined = $state();
  let inputName: HTMLInputElement | undefined = $state();

  /**
   * Update called whenever the token input changes in
   * order to normalize the casing of the input along
   * with changing the disabled state based on the
   * length requirement
   */
  function updateToken() {
    token = token
      // Convert the value to uppercase format
      .toUpperCase()
      // Remove any invalid values
      .replace(/[^A-Z0-9]/, "");

    // Enforce max token length for mobile devices
    if (token.length > TOKEN_LENGTH) {
      token = token.substring(0, TOKEN_LENGTH);
    }

    // Change the disabled state based on the length requirement
    tokenValid = token.length === TOKEN_LENGTH;
  }

  /**
   * Updates the disabled state after validating the
   * provided player name
   */
  function updateName() {
    // Change the disabled state based on the name length
    nameValid =
      name.length >= MIN_PLAYER_NAME_LENGTH &&
      name.length <= MAX_PLAYER_NAME_LENGTH;
  }

  /**
   * Handles attempting to connect to a quiz using the
   * user provided token.
   */
  function connect(event: SubmitEvent) {
    event.preventDefault();
    loading = true;

    socket
      .send({ ty: ClientMessage.Connect, token })
      .then(() => {
        connectState = State.Join;
        updateFocus = true;
      })
      .catch((error: ServerError) => {
        console.error("Failed to connect", error);
        errorDialog("Failed to connect", errorText[error]);
      })
      .finally(() => (loading = false));
  }

  function join(event: SubmitEvent) {
    event.preventDefault();
    loading = true;

    socket
      .send({ ty: ClientMessage.Join, name })
      .then(({ id, token, config }) => {
        appState.setGame({ id, token, config, host: false, name });
      })
      .catch((error: ServerError) => {
        console.error("Failed to join", error);
        errorDialog("Failed to join", errorText[error]);
      })
      .finally(() => (loading = false));
  }

  function back() {
    if (connectState === State.Connect) {
      appState.setHome();
    } else {
      connectState = State.Connect;
      inputToken?.focus();
    }
  }

  $effect(() => {
    if (!updateFocus) return;

    if (inputToken && document.activeElement !== inputToken) {
      inputToken.focus({ preventScroll: true });
    }

    if (inputName && document.activeElement !== inputName) {
      inputName.focus({ preventScroll: true });
    }
  });
</script>

{#if loading}
  <FloatingLoader />
{/if}

<button onclick={back} class="back back--floating">
  <Back />
</button>

{#if connectState === State.Connect}
  <main class="page page--center page--overflow" transition:slide|global>
    <form class="form" onsubmit={connect}>
      <h1>Enter Code</h1>
      <p>Please enter your quiz code below</p>

      <input
        bind:this={inputToken}
        class="special-input"
        type="text"
        bind:value={token}
        oninput={updateToken}
        minlength={TOKEN_LENGTH}
        maxlength={TOKEN_LENGTH}
        placeholder={"X".repeat(TOKEN_LENGTH)}
      />

      {#if tokenValid}
        <button
          type="submit"
          class="play"
          transition:slide={{ axis: "x", duration: 200 }}
        >
          <Play />
        </button>
      {/if}
    </form>
  </main>
{:else}
  <main class="page page--center page--overflow" transition:slide|global>
    <form class="form" onsubmit={join}>
      <p>{token}</p>
      <h1>Enter Name</h1>
      <p>Please enter your desired name</p>

      <input
        bind:this={inputName}
        class="special-input special-input--small"
        type="text"
        bind:value={name}
        oninput={updateName}
        minlength={MIN_PLAYER_NAME_LENGTH}
        maxlength={MAX_PLAYER_NAME_LENGTH}
      />

      {#if nameValid}
        <button
          type="submit"
          class="play play--small"
          transition:slide={{ axis: "x", duration: 200 }}
        >
          <Play />
        </button>
      {/if}
    </form>
  </main>
{/if}

<style>
  .page {
    gap: 1rem;
    min-height: 100vh;
  }

  .form {
    max-width: 32rem;
    width: 100%;
    padding: 1rem;
    text-align: center;
  }

  .special-input {
    display: inline-block;
    padding: 0.7rem;
    font-size: 3rem;
    width: 100%;
    max-width: 15rem;
    text-align: center;
    background-color: transparent;
    border: 5px solid var(--surface);
    color: var(--text-primary);
    border-radius: 0.5rem;
    outline: none;
    transition: 0.5s ease;
    letter-spacing: 0.25rem;
    vertical-align: middle;
  }

  .special-input:focus {
    border-bottom-color: var(--primary);
  }

  .special-input--small {
    font-size: 2rem;
  }

  .play {
    vertical-align: middle;
    margin-left: 1rem;
    padding: 0.8rem;
    border-radius: 1rem;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    background-color: var(--primary);
  }

  .play--small {
    padding-inline-start: 0.6rem;
  }

  .back {
    border: 0.1rem solid var(--btn-border-color);
  }

  .back :global(> svg) {
    fill: var(--text-primary);
  }

  @media screen and (max-width: 32rem) {
    .page--center {
      justify-content: flex-start;
      align-items: center;
    }

    .form {
      margin-top: 15vh;
    }

    .play,
    .special-input {
      max-width: none;
      display: block;
      margin: 1rem auto;
      width: 100%;
    }
  }
</style>
