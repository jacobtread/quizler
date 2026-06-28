<script lang="ts">
  import type { Component } from "svelte";

  interface Props {
    icon: Component;
    label: string;
    description: string;
    onClick: VoidFunction;
    "aria-label": string;
  }

  const {
    icon: Icon,
    label,
    description,
    onClick,
    "aria-label": ariaLabel
  }: Props = $props();
</script>

<button onclick={onClick} class="action" aria-label={ariaLabel}>
  <div class="action__icon">
    <Icon />
  </div>

  <div class="action__body">
    <p class="action__name">{label}</p>
    <p class="action__text">{description}</p>
  </div>
</button>

<style>
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

  .action__icon {
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

  .action:hover {
    border-color: var(--primary);
  }

  .action:hover:before {
    transform: translate(0);
  }

  .action:hover .action__icon {
    background-color: var(--primary);
  }

  @media screen and (max-width: 32rem) {
    .action__icon {
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
    .action {
      width: auto;
    }

    .action__body {
      display: none;
    }

    .action__icon {
      padding: 1rem;
      width: auto;
      margin-bottom: 0;
      margin-right: 0;
    }
  }
</style>
