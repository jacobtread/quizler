<script lang="ts">
  import { QuestionType, type Question } from "$api/models";
  import {
    activeQuestion,
    changeQuestionType,
    replaceQuestion
  } from "$lib/stores/createStore";
  import Checkbox from "../Checkbox.svelte";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import FloatingModal from "../FloatingModal.svelte";

  interface Props {
    question: Question;
    visible: boolean;
  }

  let { question = $bindable(), visible = $bindable() }: Props = $props();

  async function setQuestionType(ty: QuestionType) {
    if (ty !== question.ty) {
      const answer = await confirmDialog(
        "Confirm change",
        "Are you sure you want to change the question type? Current questions will be lost."
      );
      if (!answer) return;
    }

    question = changeQuestionType(question, ty);

    replaceQuestion(question);
    activeQuestion.set(question);
  }
</script>

<FloatingModal bind:visible>
  <div class="section">
    <h2 class="section__title">Question Type</h2>
    <p class="section__desc">Please select the type of question below</p>

    <div class="types">
      <button
        class="type"
        class:type--selected={question.ty == QuestionType.Single}
        onclick={() => setQuestionType(QuestionType.Single)}
      >
        <p class="type__name">Single Choice</p>
        <p class="type__desc">Players can only select one answer</p>
        <div class="answers">
          <p class="answer answer--correct">
            <!--  -->
          </p>
          <p class="answer">
            <!--  -->
          </p>
          <p class="answer">
            <!--  -->
          </p>
          <p class="answer">
            <!--  -->
          </p>
        </div>
      </button>

      <button
        class="type"
        class:type--selected={question.ty == QuestionType.Multiple}
        onclick={() => setQuestionType(QuestionType.Multiple)}
      >
        <p class="type__name">Multiple Choice</p>
        <p class="type__desc">Players can select multiple answers</p>
        <div class="answers">
          <p class="answer answer--correct">
            <!--  -->
          </p>
          <p class="answer answer--correct">
            <!--  -->
          </p>
          <p class="answer">
            <!--  -->
          </p>
          <p class="answer answer--correct">
            <!--  -->
          </p>
        </div>
      </button>
      <button
        class="type"
        class:type--selected={question.ty == QuestionType.TrueFalse}
        onclick={() => setQuestionType(QuestionType.TrueFalse)}
      >
        <p class="type__name">True / False</p>
        <p class="type__desc">Simple true or false questions</p>
        <div class="answers">
          <p class="answer answer--correct">
            <!--  -->
          </p>
          <p class="answer">
            <!--  -->
          </p>
        </div>
      </button>
      <button
        class="type"
        class:type--selected={question.ty == QuestionType.Typer}
        onclick={() => setQuestionType(QuestionType.Typer)}
      >
        <p class="type__name">Typer</p>
        <p class="type__desc">Players must type out their answer</p>
        <div class="answers">
          <p class="answer">
            <!--  -->
          </p>
        </div>
      </button>
    </div>
  </div>

  {#if question.ty === QuestionType.Typer}
    <div class="section">
      <h2 class="section__title">Settings</h2>
      <p class="section__desc">Below are settings specific to this type</p>
      <div>
        <div class="row">
          <Checkbox bind:value={question.ignore_case} />
          <p>Ignore case when checking if the answer is correct</p>
        </div>
      </div>
    </div>
  {/if}
</FloatingModal>

<style>
  .row {
    display: flex;
    flex-flow: row;
    gap: 0.5rem;
    align-items: center;
  }

  .types {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .type {
    text-align: left;
    background-color: var(--surface);
    border: none;
    padding: 1rem;
    border: 1px solid var(--surface-light);
    border-radius: 0.25rem;
    transition: border-color 0.25s ease;
    cursor: pointer;
  }

  .type--selected {
    border-color: var(--primary);
  }

  .type:hover {
    border-color: var(--surface-lighter);
  }

  .type--selected:hover {
    border-color: var(--primary-lighter);
  }

  .type__name {
    font-size: 1.25rem;
    font-weight: bold;
    color: var(--text-primary);
    margin-bottom: 0.25rem;
  }

  .type__desc {
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  .answers {
    overflow: hidden;
    text-align: center;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
  }

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

  @media screen and (max-width: 36rem) {
    .types {
      grid-template-columns: 1fr;
    }
  }
</style>
