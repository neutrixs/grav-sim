<script lang="ts">
  import Field from './Field.svelte'
  import RefField from './RefField.svelte'
  import { BODY_MASSES, BODY_RADII } from '../lib/units'
  import { type DraftMode, setVelocityXY, setSpeed, setAngle } from '../lib/draft'
  import { drafts } from '../state/draft.svelte'

  const hints: Record<DraftMode, string> = {
    placing: 'Click on the canvas to place it',
    aiming: 'Click again to set its velocity',
    editing: 'Drag the body or the arrow tip to adjust',
  }

  const d = $derived(drafts.draft)

  function submit(e: SubmitEvent) {
    e.preventDefault()
    drafts.confirm()
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && drafts.cancel()} />

{#if d}
  <form onsubmit={submit}>
    <h2 class="section-title">New body</h2>
    <p class="hint">{hints[drafts.mode]}</p>

    <fieldset>
      <legend>Position</legend>
      <div class="grid">
        <Field label="x" units="position" bind:value={d.x} />
        <Field label="y" units="position" bind:value={d.y} />
      </div>
    </fieldset>

    <fieldset>
      <legend>Velocity</legend>
      <div class="grid">
        <Field label="vx" units="velocity" bind:value={() => d!.vx, (v) => setVelocityXY(d!, v, d!.vy)} />
        <Field label="vy" units="velocity" bind:value={() => d!.vy, (v) => setVelocityXY(d!, d!.vx, v)} />
        <Field label="|v|" units="velocity" min={0} bind:value={() => d!.speed, (v) => setSpeed(d!, v)} />
        <Field label="θ" units="angle" bind:value={() => d!.angle, (v) => setAngle(d!, v)} />
      </div>
    </fieldset>

    <fieldset>
      <legend>Body</legend>
      <div class="stack">
        <RefField label="mass" units="mass" refs={BODY_MASSES} bind:value={d.mass} />
        <RefField label="r" units="radius" refs={BODY_RADII} bind:value={d.radius} />
      </div>
    </fieldset>

    <div class="grid">
      <button type="button" class="btn secondary" onclick={() => drafts.cancel()}>Cancel</button>
      <button type="submit" class="btn">Add</button>
    </div>

    {#if drafts.error}<p class="error">{drafts.error}</p>{/if}
  </form>
{/if}

<style>
  .hint {
    margin: -4px 0 12px;
    font-size: 12px;
    color: var(--accent);
  }

  fieldset {
    margin: 0 0 12px;
    padding: 0;
    border: none;
    min-width: 0;
  }
  legend {
    padding: 0;
    margin-bottom: 6px;
    font-size: 11px;
    color: var(--text-dim);
  }

  .grid {
    display: grid;
    /* minmax(0, …) lets columns shrink below their content's width */
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }
  /* Body section: mass and radius need the full width for their presets */
  .stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .error {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--error);
  }
</style>
