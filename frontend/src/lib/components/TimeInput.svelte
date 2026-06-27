<!-- Input for deciding the amount of time something should take -->

<script lang="ts">
  interface Props {
    // The time in milliseconds for the input
    value: number;
    min: number;
    max: number;
  }

  let { value = $bindable(), min, max }: Props = $props();

  const Unit = {
    Seconds: 0,
    Minutes: 1
  };

  let unit = $state(Unit.Seconds);
  let actualValue = $state(convertTo(value));

  function convertFrom(value: number) {
    if (unit == Unit.Seconds) {
      return value * 1000;
    } else if (unit == Unit.Minutes) {
      return value * 1000 * 60;
    } else {
      return value;
    }
  }

  function convertTo(value: number) {
    if (unit == Unit.Seconds) {
      return value / 1000;
    } else if (unit == Unit.Minutes) {
      return value / 60 / 1000;
    } else {
      return value;
    }
  }

  /**
   * Handles updating the produced value to ensure it
   * stays within the desired range
   */
  function updateValue() {
    if (actualValue < 1) {
      actualValue = 1;
    } else {
      let converted = convertFrom(actualValue);
      if (converted > max) {
        converted = max;
      } else if (converted < min) {
        converted = min;
      }
      actualValue = convertTo(converted);
      value = converted;
    }
  }
</script>

<div>
  <input
    class="time-input"
    type="number"
    min={convertTo(min)}
    max={convertTo(max)}
    bind:value={actualValue}
    onchange={updateValue}
  />
  <select class="select" bind:value={unit} onchange={updateValue}>
    <option value={Unit.Minutes}>Minutes</option>
    <option value={Unit.Seconds}>Seconds</option>
  </select>
</div>

<style>
  .select,
  .time-input {
    padding: 0.5rem;
    border-radius: 0.25rem;
    font-size: 1rem;
    background-color: var(--btn-surface-background);
    color: var(--btn-text);
    border: none;
  }

  .time-input {
    margin-right: 0.25rem;
  }

  .select {
    cursor: pointer;
  }
</style>
