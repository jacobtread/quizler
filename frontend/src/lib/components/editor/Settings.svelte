<script lang="ts">
  import { NameFiltering, nameFilterText } from "$api/models";
  import * as constants from "$lib/constants";

  import { createData } from "$stores/createStore";
  import FloatingModal from "../FloatingModal.svelte";

  interface Props {
    visible: boolean;
  }

  let { visible = $bindable() }: Props = $props();
</script>

<FloatingModal bind:visible>
  <div class="section">
    <div class="section__values">
      <label class="field">
        <span class="field__name">Title</span>
        <p class="field__desc">
          Give your quiz a title <span class="optional">Optional</span>
        </p>
        <input
          class="input"
          type="text"
          bind:value={$createData.name}
          maxlength={constants.MAX_TITLE_LENGTH}
        />
      </label>
      <label class="field">
        <span class="field__name">Description</span>
        <p class="field__desc">
          Description of your Quiz <span class="optional">Optional</span>
        </p>
        <textarea
          class="input input--desc"
          name=""
          id=""
          cols="40"
          rows="2"
          bind:value={$createData.text}
          maxlength={constants.MAX_DESCRIPTION_LENGTH}></textarea>
      </label>
    </div>
  </div>

  <div class="section">
    <div class="section__values">
      <label class="field">
        <span class="field__name">Max Players</span>
        <p class="field__desc">
          Maximum number of players allowed to join this quiz
        </p>
        <input
          class="input"
          type="number"
          bind:value={$createData.max_players}
          min={constants.MIN_MAX_PLAYERS}
          max={constants.MAX_MAX_PLAYERS}
        />
      </label>
      <label class="field">
        <span class="field__name">Name Filtering</span>
        <p class="field__desc">
          Level of filtering on profane/inappropriate naming. Its recommended
          that you leave this on <b>Medium</b> or <b>High</b>
        </p>
        <select bind:value={$createData.filtering} class="input">
          {#each Object.values(NameFiltering) as value (value)}
            <option {value}>{value}: {nameFilterText[value]}</option>
          {/each}
        </select>
      </label>
    </div>
  </div>
</FloatingModal>

<style lang="scss">
  @use "../../../assets/scheme.scss";

  .optional {
    color: scheme.$textHint;
    margin-left: 0.5rem;
  }

  textarea {
    resize: none;
  }
</style>
