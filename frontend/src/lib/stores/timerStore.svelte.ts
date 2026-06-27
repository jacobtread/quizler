export interface TimerStore {
  /**
   * Get the remaining time in milliseconds
   * of  the timer
   */
  readonly current: number;

  /**
   * Start a the timer with a new duration
   *
   * @param duration The duration of the timer in milliseconds
   */
  start(duration: number): void;

  /**
   * Reset the timer
   */
  reset(): void;
}

/**
 * Creates a store for tracking time passed for a timer.
 *
 * Time is ticked using animation frames and performance.now()
 * for smooth timing updates
 *
 * @returns The timer store
 */
export function createTimerStore(): TimerStore {
  let timeMs: number = $state(0);
  let lastUpdateTime: number = 0;

  function tick() {
    // Don't update the timer if we have reached the time
    if (timeMs <= 0) return;

    const time = performance.now();

    const elapsed = time - lastUpdateTime;

    timeMs -= elapsed;
    if (timeMs < 0) timeMs = 0;

    lastUpdateTime = time;

    if (timeMs != 0) {
      // Request the next animation frame
      requestAnimationFrame(tick);
    }
  }

  function start(duration: number) {
    lastUpdateTime = performance.now();
    timeMs = duration;
    tick();
  }

  function reset() {
    lastUpdateTime = performance.now();
    timeMs = 0;
  }

  return {
    get current() {
      return timeMs;
    },
    start,
    reset
  };
}
