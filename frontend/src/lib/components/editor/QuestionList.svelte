<script lang="ts">
  import { flip } from "svelte/animate";

  import { dndzone, type DndEvent } from "svelte-dnd-action";

  import { type Question } from "$api/models";
  import * as constants from "$lib/constants";

  import QuestionListItem from "$lib/components/editor/QuestionListItem.svelte";
  import Add from "$components/icons/Add.svelte";
  import Shuffle from "$components/icons/Shuffle.svelte";

  import {
    createData,
    shuffleQuestions,
    addQuestion,
    activeQuestion
  } from "$stores/createStore";

  function handleDnd(e: CustomEvent<DndEvent<Question>>) {
    $createData.questions = e.detail.items;
  }

  function onClickQuestion(question: Question) {
    activeQuestion.set(question);
  }

  function onKeyQuestion(event: KeyboardEvent, question: Question) {
    if (event.key === "Return" || event.key == "Enter") {
      activeQuestion.set(question);
    }
  }
</script>

<div class="list">
  <button
    onclick={shuffleQuestions}
    disabled={$createData.questions.length <= 1}
    class="btn btn--icon"
  >
    <Shuffle />
    Shuffle
  </button>
  <section
    class="questions"
    use:dndzone={{
      items: $createData.questions as Question[],
      flipDurationMs: 200,
      dropTargetStyle: {}
    }}
    onfinalize={handleDnd}
    onconsider={handleDnd}
  >
    {#each $createData.questions as question, index (question.id)}
      <div
        tabindex="0"
        class="qw"
        animate:flip={{ duration: 200 }}
        class:qw--active={$activeQuestion !== null &&
          $activeQuestion.id === question.id}
        onclick={() => onClickQuestion(question)}
        onkeydown={(event) => onKeyQuestion(event, question)}
        role="button"
      >
        <QuestionListItem {question} {index} />
      </div>
    {/each}
  </section>
  <button
    onclick={addQuestion}
    disabled={$createData.questions.length >= constants.MAX_QUESTIONS}
    class="btn add btn--icon-only"
  >
    <Add />
  </button>
</div>

<style>
  .qw {
    border-radius: 0.25rem;
    position: relative;
    border: 2px solid var(--surface);
  }

  .qw:hover {
    border-color: var(--surface-lightest);
  }

  .qw--active,
  .qw--active:hover {
    border-color: var(--primary);
  }

  .qw:focus {
    outline: 2px solid var(--outline-fill);
    outline-offset: 4px;
  }

  .list {
    display: flex;
    flex-flow: column;
    gap: 1rem;

    min-width: 12rem;
  }

  .questions {
    position: relative;
    padding: 0.75rem;
    overflow: auto;
    flex: auto;
    border: 0.1rem solid var(--surface-light);
    border-radius: 0.5rem;

    display: flex;
    gap: 0.75rem;
    flex-flow: column;
    list-style: none;
  }

  @media screen and (max-width: 64rem) {
    .list {
      flex-flow: row;
      width: auto;
    }

    .questions {
      flex-flow: row;
    }
  }

  @media screen and (max-width: 64rem), (max-height: 48rem) {
    .questions {
      align-items: center;
      padding: 0;

      border: none;
    }
  }
</style>
