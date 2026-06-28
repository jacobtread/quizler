<!-- Preview image display for a question -->
<script lang="ts">
  import { imagePreviewStore } from "$lib/stores/imageStore";
  import type { ImageFit } from "$lib/api/models";

  interface Props {
    // UUID based preview image loading
    uuid?: string | null;
    // Preloaded question images
    preloaded?: HTMLImageElement | null;
    // Image fitting
    fit: ImageFit;
  }

  let { uuid = null, preloaded = null, fit }: Props = $props();

  const src = $derived.by(() => {
    // Handle displaying image previews by UUID from the preview store
    if (uuid !== null) {
      let imagePreview = $imagePreviewStore[uuid];
      return imagePreview ?? null;
    }

    // Handle displaying preloaded images
    // (The src is already loaded, decoded, and cached in browser memory this image loads instantly)
    if (preloaded !== null) {
      return preloaded.src;
    }

    return null;
  });
</script>

<div class="qu-image-wrapper">
  {#if src !== null}
    <img class="qu-image" data-fit={fit} {src} alt="Uploaded Content" />
  {/if}
</div>

<style>
  /* Wrapper for custom sized images */
  .qu-image-wrapper {
    position: relative;

    width: 100%;
    height: 100%;

    overflow: hidden;
  }

  .qu-image {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    aspect-ratio: auto;
  }

  /* Fit for width */
  .qu-image[data-fit="Width"] {
    width: 100%;
  }

  /* Fit for height */
  .qu-image[data-fit="Height"] {
    height: 100%;
  }

  /* Fit for containing whole image */
  .qu-image[data-fit="Contain"] {
    height: 100%;
    width: 100%;
    object-fit: contain;
  }

  /* Fit for covering available space */
  .qu-image[data-fit="Cover"] {
    height: 100%;
    width: 100%;
    object-fit: cover;
  }
</style>
