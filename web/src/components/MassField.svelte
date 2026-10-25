<script lang="ts">
  import Field from './Field.svelte'
  import { type Unit, BODY_MASSES } from '../lib/units'
  import { unitPrefs } from '../state/units.svelte'

  // Mass field with reference bodies for a sense of scale: a chip sets the
  // mass to exactly that body's, and the line below says which reference the
  // current mass is closest to.
  let { value = $bindable() }: { value: number } = $props()

  function pick(ref: Unit) {
    value = ref.factor
    unitPrefs.mass = ref.label
  }

  // Closest reference on a log scale, if the mass is within 1000× of it.
  const nearest = $derived.by(() => {
    if (!(value > 0)) return null
    let best = BODY_MASSES[0]
    let bestDist = Infinity
    for (const ref of BODY_MASSES) {
      const dist = Math.abs(Math.log10(value / ref.factor))
      if (dist < bestDist) {
        best = ref
        bestDist = dist
      }
    }
    return bestDist < 3 ? best : null
  })
</script>

<div class="mass">
  <Field label="mass" units="mass" bind:value />

  <div class="refs">
    {#each BODY_MASSES as ref (ref.label)}
      <button type="button" class:active={value === ref.factor} onclick={() => pick(ref)}>{ref.label}</button>
    {/each}
  </div>

  {#if nearest && unitPrefs.mass !== nearest.label}
    <p class="near">≈ {Number((value / nearest.factor).toPrecision(3))} × {nearest.label}</p>
  {/if}
</div>

<style>
  .mass {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  /* Reference body presets */
  .refs {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 4px;
  }
  .refs button {
    padding: 3px 0;
    font: inherit;
    font-size: 11px;
    color: var(--text-dim);
    background: var(--input);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
  }
  .refs button:hover {
    color: var(--text);
    border-color: var(--accent);
  }
  .refs button.active {
    color: var(--bg);
    background: var(--accent);
    border-color: var(--accent);
  }

  .near {
    margin: 0;
    font: 11px var(--mono);
    color: var(--text-dim);
  }
</style>
