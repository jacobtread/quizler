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
  import TypePreview from "./type-preview/TypePreview.svelte";

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

  const types = [
    {
      type: QuestionType.Single,
      name: "Single Choice",
      description: "Players can only select one answer",
      answers: [true, false, false, false]
    },
    {
      type: QuestionType.Multiple,
      name: "Multiple Choice",
      description: "Players can select multiple answers",
      answers: [true, true, false, true]
    },
    {
      type: QuestionType.TrueFalse,
      name: "True / False",
      description: "Simple true or false questions",
      answers: [true, false]
    },
    {
      type: QuestionType.Typer,
      name: "Typer",
      description: "Players must type out their answer",
      answers: [false]
    }
  ];
</script>

<FloatingModal bind:visible>
  <div class="section">
    <h2 class="section__title">Question Type</h2>
    <p class="section__desc">Please select the type of question below</p>

    <div class="types">
      {#each types as type (type)}
        <TypePreview
          selected={question.ty === type.type}
          onClick={() => setQuestionType(type.type)}
          name={type.name}
          description={type.description}
          answers={type.answers}
        />
      {/each}
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

  @media screen and (max-width: 36rem) {
    .types {
      grid-template-columns: 1fr;
    }
  }
</style>
