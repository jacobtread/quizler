<script lang="ts">
  interface Props {
    correct?: boolean;
  }

  const { correct }: Props = $props();
</script>

<p class="answer" class:answer--correct={correct}>
  <!--  -->
</p>

<style>
  .answer {
    padding: 0.5rem;
    border-radius: 0.25rem;
    background-color: var(--surface-light);
    transition: background-color 0.1s linear;
  }

  .answer:nth-child(odd):last-child {
    grid-column-start: 1;
    grid-column-end: 3;
  }

  .answer--correct {
    background-color: var(--primary);
    color: var(--text-primary);
  }
</style>
