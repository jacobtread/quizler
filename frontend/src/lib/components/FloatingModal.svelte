<script module lang="ts">
  export const ModelSize = {
    Normal: 0,
    Small: 1
  } as const;

  type ModelSizeType = (typeof ModelSize)[keyof typeof ModelSize];
</script>

<script lang="ts">
  import { fade, slide } from "svelte/transition";
  import Close from "$assets/icons/delete.svg?component";
  import type { Snippet } from "svelte";

  interface Props {
    visible: boolean;
    size?: ModelSizeType;
    children?: Snippet;
  }

  let {
    visible = $bindable(),
    size = ModelSize.Normal,
    children
  }: Props = $props();
</script>

{#if visible}
  <div class="floating-wrapper" transition:fade={{ duration: 200 }}>
    <div
      class="dialog"
      class:dialog--small={size == ModelSize.Small}
      transition:slide|global={{ duration: 250 }}
    >
      <button onclick={() => (visible = false)} class="btn btn--icon">
        <Close />
        Close
      </button>

      {@render children?.()}
    </div>
  </div>
{/if}

<style>
  .floating-wrapper {
    z-index: 1;
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }

  .dialog {
    background-color: var(--app-background);
    border: 1px solid var(--surface);

    border-radius: 0.5rem;

    width: 100%;
    max-width: 48rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .dialog--small {
    max-width: 32rem;
  }

  @media screen and (max-width: 48rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }
</style>
