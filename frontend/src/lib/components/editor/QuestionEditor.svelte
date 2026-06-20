<!-- Component represents a question that is being created -->

<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import * as constants from "$lib/constants";

  import { shuffleArray } from "$lib/utils/utils";

  import QuestionSettings from "$components/editor/QuestionSettings.svelte";
  import QuestionTypeSelect from "$components/editor/QuestionTypeSelect.svelte";
  import EditorAnswers from "$components/editor/EditorAnswers.svelte";
  import ImageEditor from "$components/editor/ImageEditor.svelte";
  import Shuffle from "$components/icons/Shuffle.svelte";
  import Delete from "$components/icons/Delete.svelte";
  import Swap from "$components/icons/Swap.svelte";
  import Cog from "$components/icons/Cog.svelte";

  import { removeQuestion } from "$stores/createStore";
  import { confirmDialog } from "$stores/dialogStore";

  interface Props {
    question: Question;
  }

  let { question = $bindable() }: Props = $props();

  let settings: boolean = $state(false);
  let type: boolean = $state(false);

  function shuffleAnswers() {
    if (
      question.ty != QuestionType.Single &&
      question.ty != QuestionType.Multiple
    ) {
      return;
    }
    question.answers = shuffleArray(question.answers);
  }

  // Remove the question
  async function remove() {
    const confirmed = await confirmDialog(
      "Confirm Deletion",
      "Are you sure you want to delete this question?"
    );

    if (!confirmed) return;

    removeQuestion(question);
  }
</script>

<ImageEditor bind:question />

<div class="actions btn-row">
  <button onclick={() => (settings = true)} class="btn btn--icon">
    <Cog />
    <span>Settings</span>
  </button>

  <!-- If question type can be shuffled include a shuffle button -->
  {#if (question.ty === QuestionType.Single || question.ty === QuestionType.Multiple) && question.answers !== undefined}
    <button
      class="btn btn--icon"
      onclick={shuffleAnswers}
      disabled={question.answers.length <= 1}
    >
      <Shuffle />
      <span>Shuffle Answers</span>
    </button>
  {/if}

  <button class="btn btn--icon qt" onclick={() => (type = true)}>
    <Swap />
    <span>Change Type</span>

    <span class="qt__type">{question.ty}</span>
  </button>

  <button onclick={remove} class="btn btn--icon">
    <Delete />
    <span>Delete</span>
  </button>
</div>

<textarea
  class="text input"
  cols="30"
  rows="2"
  maxlength={constants.MAX_QUESTION_LENGTH}
  bind:value={question.text}>
  <!-- -->
</textarea>

<EditorAnswers bind:question />
<QuestionSettings bind:question bind:visible={settings} />
<QuestionTypeSelect bind:question bind:visible={type} />

<style lang="scss">
  @use "../../../assets/scheme.scss";

  .actions {
    margin-bottom: 0.5rem;
    display: flex;
    flex-flow: row wrap;
  }

  .qt {
    padding: 0.5rem;

    &__type {
      background-color: scheme.$surfaceLight;
      padding: 0.5rem;
      border-radius: 0.5rem;
      margin-left: 0.5rem;
    }
  }

  .text {
    display: block;
    width: 100%;
    resize: vertical;

    margin-bottom: 1rem;

    min-height: 5rem;
    max-height: 6rem;

    padding: 0.5rem;
    border: none;
    background-color: scheme.$surface;
    border-radius: 0.25rem;
    margin-top: 0.5rem;
    font-size: 1rem;
    line-height: 1.5;
    color: scheme.$textDefault;
  }

  @media screen and (max-width: 64rem) {
    .actions {
      .btn {
        flex: auto;
      }
    }
  }

  @media screen and (max-width: 48rem) {
    .btn > span {
      display: none;
    }
  }
</style>
