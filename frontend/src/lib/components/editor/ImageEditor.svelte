<script lang="ts">
  import { ImageFit, imageFitText, type Question } from "$api/models";
  import { selectImage } from "$stores/imageStore";

  import FloatingModal, { ModelSize } from "$components/FloatingModal.svelte";
  import QuPreviewImage from "$components/editor/QuPreviewImage.svelte";
  import ImageStorage from "$components/ImageStorage.svelte";
  import Cog from "$components/icons/Cog.svelte";

  interface Props {
    question: Question;
  }

  let { question = $bindable() }: Props = $props();

  let settings: boolean = $state(false);

  async function pickImage() {
    let res = await selectImage();
    // Handle canceling select image
    if (res === null) return;

    question.image = {
      uuid: res.uuid,
      fit: ImageFit.Contain
    };
  }

  function removeImage() {
    question.image = null;
  }
</script>

{#if question.image !== null}
  <div class="wrapper">
    <!-- Actual preview image may not be immediately available -->
    <QuPreviewImage uuid={question.image.uuid} fit={question.image.fit} />
    <button class="overlay" onclick={removeImage}>Click to remove</button>
    <button
      class="btn btn--icon btn--icon-only settings"
      onclick={() => (settings = true)}
    >
      <Cog />
    </button>
  </div>

  <!-- Dialog for changing image settings -->
  <FloatingModal bind:visible={settings} size={ModelSize.Small}>
    <label class="section">
      <h2 class="section__title">Image Fit</h2>
      <p class="section__desc">
        How the image should be fit to devices. It's recommended that you use
        <b>Contain</b> if its important that they can see the whole image
      </p>

      <select class="input" bind:value={question.image.fit}>
        {#each Object.values(ImageFit) as value (value)}
          <option {value}>{value}: {imageFitText[value]}</option>
        {/each}
      </select>
    </label>
  </FloatingModal>
{:else}
  <div class="wrapper">
    <button class="add" onclick={pickImage}>Pick Image</button>
  </div>
{/if}

<!-- Image store access for rendering the image picker -->
<ImageStorage />

<style>
  .settings {
    position: absolute;
    right: 0.5rem;
    top: 0.5rem;
  }

  .wrapper {
    width: 100%;
    flex: auto;
    overflow: hidden;
    position: relative;
    margin-bottom: 1rem;

    border: 1px solid var(--surface-light);
    border-radius: 0.5rem;
  }

  @media screen and (max-width: 64rem), (max-height: 48rem) {
    .wrapper {
      height: 50vh;
      min-height: 20rem;
    }
  }

  .add {
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    font-size: 1rem;
    background-color: transparent;
    border: none;
    border-radius: 0.5rem;
    color: var(--text-default);
  }

  .overlay {
    cursor: pointer;
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    transition: opacity 0.15s ease;
    font-size: 1rem;
    background-color: rgba(0, 0, 0, 0.7);
    border: none;
    opacity: 0;
    border-radius: 0.5rem;
  }

  .overlay:hover,
  .overlay:focus {
    opacity: 1;
  }
</style>
