<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import { flip } from "svelte/animate";
  import Image from "$assets/icons/image.svg?component";
  import { SHADOW_ITEM_MARKER_PROPERTY_NAME } from "svelte-dnd-action";
  import QuPreviewImage from "./QuPreviewImage.svelte";

  interface Props {
    question: Question;
    index: number;
  }

  const { question, index }: Props = $props();
</script>

<div class="question">
  <div class="image-wrapper">
    {#if question.image !== null}
      <QuPreviewImage uuid={question.image.uuid} fit={question.image.fit} />
    {:else}
      <Image />
    {/if}
  </div>
  <p class="text">
    <span class="question__index">{index + 1} </span>
    {question.text}
  </p>

  {#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
    <div class="answers">
      {#each question.answers as answer (answer.id)}
        <p
          animate:flip={{ duration: 250 }}
          class="answer"
          data-correct={answer.correct}
        >
          <!--  -->
        </p>
      {/each}
    </div>
  {:else if question.ty === QuestionType.TrueFalse}
    <div class="answers">
      <p class="answer" data-correct={question.answer === true}>
        <!--  -->
      </p>
      <p class="answer" data-correct={question.answer === false}>
        <!--  -->
      </p>
    </div>
  {:else if question.ty === QuestionType.Typer}
    <p class="answer" data-correct={false}>
      <!--  -->
    </p>
  {/if}
</div>

<!-- Shadow item for drag and drop -->
{#if question[SHADOW_ITEM_MARKER_PROPERTY_NAME]}
  <div class="shadow">
    <!--  -->
  </div>
{/if}

<style>
  .image-wrapper {
    width: 100%;
    height: 3rem;
    overflow: hidden;
    position: relative;

    display: flex;
    justify-content: center;
    align-items: center;

    border: 1px solid var(--surface-light);
    border-radius: 0.25rem;

    color: #666666;
  }

  .question {
    position: relative;
    background-color: var(--surface);
    padding: 0.5rem;
    border-radius: 0.25rem;
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
    width: 12rem;
  }

  .question__index {
    display: inline;
    background-color: var(--surface);

    color: var(--text-primary);

    font-weight: bold;
    margin-right: 0.25rem;
  }

  .text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.8rem;
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

  .answer[data-correct="true"] {
    background-color: var(--primary);
    color: var(--text-primary);
  }

  .shadow {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    visibility: visible;
    border: 2px dashed var(--surface-light);
    border-radius: 0.25rem;
    margin: 0;
  }

  @media screen and (max-width: 64rem) {
    .question {
      max-width: 12rem;
      flex-flow: row;
      align-items: center;
      padding: 0.2rem;
    }

    .answers {
      display: none;
    }
  }
</style>
