<script lang="ts">
  import Back from "$assets/icons/back.svg?component";

  interface Props {
    onClick: VoidFunction;
  }

  const { onClick }: Props = $props();
</script>

<button onclick={onClick} class="back">
  <Back />
</button>

<style>
  .back {
    padding: 0.5rem;
    border-radius: 0.5rem;

    background-color: var(--btn-background);
    border: 0.1rem solid var(--btn-border-color);
    color: var(--text-primary);

    transition:
      background-color 0.25s ease,
      transform 0.2s ease;
    line-height: 0;
    cursor: pointer;

    z-index: 1;

    position: absolute;
    left: 1rem;
    top: 1rem;
  }

  .back:hover {
    background-color: var(--primary);
  }
</style>
