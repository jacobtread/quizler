<div class="spin">
  <!--  -->
</div>

<style>
  .spin {
    width: 4rem;
    height: 4rem;
    display: block;
    border: 5px solid var(--spinner-fill);
    border-radius: 2rem;
    border-top-color: transparent;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    0% {
      transform: rotate(0);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
