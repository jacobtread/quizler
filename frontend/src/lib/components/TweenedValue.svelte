<!-- Number component that produced a value that is tweened to its target -->
<script lang="ts">
  import { Tween } from "svelte/motion";

  interface Props {
    // The actual value
    value: number;
  }

  let { value }: Props = $props();

  // Create a tweened version of the value
  const valueTween = Tween.of(() => value, { delay: 500 });
</script>

{valueTween.current.toFixed(0)}
