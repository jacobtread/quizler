<script lang="ts">
  import { confirmDialog } from "$stores/dialogStore";
  import {
    imageStore,
    selectImageStore,
    uploadFile,
    type StoredImage,
    clearSelectImage,
    consumeSelectImage,
    imagePreviewStore
  } from "$stores/imageStore";
  import QuPreviewImage from "$components/editor/QuPreviewImage.svelte";
  import Import from "$assets/icons/import.svg?component";
  import { acceptUploadMany } from "$lib/utils/file";
  import { formatBytes } from "$lib/utils/utils";
  import { ImageFit } from "$api/models";
  import { fade, slide } from "svelte/transition";

  let uploading: FileUpload[] = $state([]);

  interface FileUpload {
    name: string;
    progress: number;
    error: string | null;
  }

  async function doUpload() {
    const files: FileList | null = await acceptUploadMany("image/*");

    // No files were uploaded
    if (files === null) return;

    uploadFiles(files);
  }

  function onDrop(event: DragEvent) {
    event.stopPropagation();
    event.preventDefault();

    const dataTransfer = event.dataTransfer;
    if (dataTransfer === null) return;
    const files = dataTransfer.files;
    if (files === null) return;
    uploadFiles(files);
  }

  function onDragOver(event: DragEvent) {
    event.stopPropagation();
    event.preventDefault();
  }

  function uploadFiles(files: FileList) {
    for (const file of files) {
      uploading.push({
        name: file.name,
        progress: 0,
        error: null
      });

      uploading = uploading;
      uploadFile(file, (progress) => {
        onProgress(file.name, progress);
      })
        .then(() => onUploadComplete(file))
        .catch((error) => onUploadFailed(file, error));
    }
  }

  function onUploadComplete(file: File) {
    uploading = uploading.filter((value) => value.name !== file.name);
  }

  function onUploadFailed(file: File, error: Error) {
    for (const value of uploading) {
      if (value.name === file.name) {
        value.error = error.message;
        break;
      }
    }
    uploading = uploading;
  }

  function onProgress(name: string, progress: number) {
    for (const value of uploading) {
      if (value.name === name) {
        value.progress = progress;
        break;
      }
    }
    uploading = uploading;
  }

  function onDelete(image: StoredImage) {
    // Remove the image from the image store
    imageStore.update((store) => {
      return store.filter((value) => value.uuid !== image.uuid);
    });

    // Remove the image preview
    imagePreviewStore.update((store) => {
      delete store[image.uuid];
      return store;
    });
  }

  async function doClear() {
    const confirmed = await confirmDialog(
      "Confirm Deletion",
      "Are you sure you want to delete all the uploaded files?"
    );

    if (!confirmed) return;

    imageStore.set([]);
    imagePreviewStore.set({});
    uploading = [];
  }
</script>

{#if $selectImageStore}
  <div class="wrapper" transition:fade={{ duration: 200 }}>
    <div
      class="dialog"
      ondrop={onDrop}
      ondragover={onDragOver}
      aria-hidden="true"
      transition:slide|global={{ duration: 250 }}
    >
      <div class="images">
        <!-- Don't show the upload info if already uploading -->
        {#if $imageStore.length < 1 && uploading.length < 1}
          <p class="images__text">
            Click upload or drag and drop files here to upload
          </p>
        {/if}

        {#each $imageStore as image (image.uuid)}
          <div class="file">
            <p class="file__name">{image.name}</p>
            <div class="image-wrapper">
              <QuPreviewImage uuid={image.uuid} fit={ImageFit.Cover} />
            </div>
            <p>{formatBytes(image.size)}</p>
            <div class="file__actions">
              <button
                class="btn btn--small"
                onclick={() => consumeSelectImage(image)}>Select</button
              >
              <button class="btn btn--small" onclick={() => onDelete(image)}>
                Delete
              </button>
            </div>
          </div>
        {/each}

        {#each uploading as upload, i (i)}
          <div class="file">
            <p class="file__name">{upload.name}</p>
            <div class="image-wrapper">
              <p class="file__progress">
                Progress: {upload.progress.toFixed(0)}%
              </p>
            </div>
            {#if upload.error}
              <p class="error">{upload.error}</p>
            {/if}
          </div>
        {/each}
      </div>

      <div class="actions btn-row btn-row--fill">
        <button onclick={clearSelectImage} class="btn">Close</button>
        <button onclick={doUpload} class="btn btn--icon">
          <Import />
          Upload Images
        </button>
        <button
          onclick={doClear}
          class="btn"
          disabled={$imageStore.length === 0}>Delete All Images</button
        >
      </div>
    </div>
  </div>
{/if}

<style>
  .error {
    color: var(--text-error);
  }

  .wrapper {
    position: fixed;
    z-index: 1;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;

    display: flex;
    justify-content: center;
    align-items: center;

    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }

  .dialog {
    background-color: var(--app-background);
    border: 1px solid var(--surface);

    padding: 1rem;
    border-radius: 0.5rem;
    max-width: 48rem;
    width: 100%;
    margin: 1rem;
  }

  .images {
    display: grid;

    grid-template-columns: repeat(4, minmax(0, 1fr));
    grid-auto-rows: min-content;
    gap: 0.5rem;

    padding: 0.5rem;
    border-radius: 0.25rem;
    background-color: var(--surface);

    height: 60vh;

    overflow: auto;
    margin-bottom: 1rem;
    position: relative;
  }

  .images__text {
    position: absolute;
    left: 50%;
    top: 50%;
    max-width: 30%;
    text-align: center;
    transform: translate(-50%, -50%);
  }

  .image-wrapper {
    width: 100%;
    height: 80px;
    position: relative;
    border: 1px solid var(--surface);
    background-color: var(--surface-light);
  }

  .file {
    border: 1px solid var(--surface-light);
    background-color: var(--app-background);

    padding: 0.5rem;
    border-radius: 0.25rem;
  }

  .file__name {
    text-overflow: ellipsis;
    white-space: nowrap;
    overflow: hidden;
    color: var(--text-primary);
    font-weight: bold;
    margin-bottom: 0.5rem;
  }

  .file__actions {
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .file__progress {
    padding: 0.5rem;
    text-align: center;
  }

  .actions {
    display: flex;
    gap: 1rem;
  }

  @media screen and (max-width: 48rem) {
    .images {
      grid-template-columns: 1fr 1fr;
    }
  }

  @media screen and (max-width: 32rem) {
    .images {
      grid-template-columns: 1fr;
    }

    .actions {
      display: flex;
      flex-flow: column;
    }
  }
</style>
