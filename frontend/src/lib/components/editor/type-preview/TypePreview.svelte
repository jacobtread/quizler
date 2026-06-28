<script lang="ts">
  import TypePreviewAnswer from "./TypePreviewAnswer.svelte";

  interface Props {
    selected: boolean;
    onClick: VoidFunction;

    name: string;
    description: string;
    answers: boolean[];
  }

  const { selected, onClick, name, description, answers }: Props = $props();
</script>

<button class="type" class:type--selected={selected} onclick={onClick}>
  <p class="type__name">{name}</p>
  <p class="type__desc">{description}</p>
  <div class="type__answers">
    {#each answers as answer, index (index)}
      <TypePreviewAnswer correct={answer} />
    {/each}
  </div>
</button>

<style>
  .type {
    text-align: left;
    background-color: var(--surface);
    border: none;
    padding: 1rem;
    border: 1px solid var(--surface-light);
    border-radius: 0.25rem;
    transition: border-color 0.25s ease;
    cursor: pointer;
  }

  .type--selected {
    border-color: var(--primary);
  }

  .type:hover {
    border-color: var(--surface-lighter);
  }

  .type--selected:hover {
    border-color: var(--primary-lighter);
  }

  .type__name {
    font-size: 1.25rem;
    font-weight: bold;
    color: var(--text-primary);
    margin-bottom: 0.25rem;
  }

  .type__desc {
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  .type__answers {
    overflow: hidden;
    text-align: center;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
  }
</style>
