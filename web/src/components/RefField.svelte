<script lang="ts">
  import Field from './Field.svelte'
  import type { Unit } from '../lib/units'
  import { unitPrefs } from '../state/units.svelte'

  // Field with reference bodies for a sense of scale (Moon … Sun): a chip sets
  // the value to exactly that body's, and the line below says which reference
  // the current value is closest to. Used for mass and radius.
  interface Props {
    label: string
    units: 'mass' | 'radius'
    refs: Unit[] // also present in the group's unit list, under the same labels
    value: number
  }
  let { label, units, refs, value = $bindable() }: Props = $props()

  function pick(ref: Unit) {
    value = ref.factor
    unitPrefs[units] = ref.label
  }

  // Closest reference on a log scale, if the value is within 1000× of it.
  const nearest = $derived.by(() => {
    if (!(value > 0)) return null
    let best = refs[0]
    let bestDist = Infinity
    for (const ref of refs) {
      const dist = Math.abs(Math.log10(value / ref.factor))
      if (dist < bestDist) {
        best = ref
        bestDist = dist
      }
    }
    return bestDist < 3 ? best : null
  })
</script>

<div class="ref-field">
  <Field {label} {units} bind:value />

  <div class="refs">
    {#each refs as ref (ref.label)}
      <button type="button" class:active={value === ref.factor} onclick={() => pick(ref)}>{ref.label}</button>
    {/each}
  </div>

  {#if nearest && unitPrefs[units] !== nearest.label}
    <p class="near">≈ {Number((value / nearest.factor).toPrecision(3))} × {nearest.label}</p>
  {/if}
</div>

<style>
  .ref-field {
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
