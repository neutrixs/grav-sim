<script lang="ts">
  import { type UnitGroup, UNIT_GROUPS, toDisplay, fromDisplay } from '../lib/units'
  import { unitPrefs, unitOf } from '../state/units.svelte'

  // Number input laid out as [label | value | unit] in one box.
  // With `units`, the unit is a picker and `value` is always in the stored
  // unit (the field converts for display). With `unit`, it's a fixed label.
  interface Props {
    label: string
    units?: UnitGroup
    unit?: string
    value: number
    min?: number
  }
  let { label, units, unit: fixedUnit, value = $bindable(), min }: Props = $props()

  const unit = $derived(units ? unitOf(units) : { label: fixedUnit ?? '', factor: 1 })
</script>

<label class="field">
  <span class="tag">{label}</span>
  <input
    type="number"
    step="any"
    {min}
    bind:value={() => toDisplay(value, unit), (v) => (value = fromDisplay(v, unit))}
  />
  {#if units}
    <span class="unit">
      <select bind:value={unitPrefs[units]} aria-label="{label} unit">
        {#each UNIT_GROUPS[units] as u (u.label)}
          <option value={u.label}>{u.label}</option>
        {/each}
      </select>
      <span class="caret">▾</span>
    </span>
  {:else if fixedUnit}
    <span class="tag">{fixedUnit}</span>
  {/if}
</label>

<style>
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 6px 0 8px;
    box-sizing: border-box;
    background: var(--input);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: text;
  }
  .field:focus-within {
    border-color: var(--accent);
  }

  .tag {
    flex: none;
    font: 11px var(--mono);
    color: var(--text-dim);
    user-select: none;
  }

  input {
    /* width: 0 + flex: 1 → take only the space left by the tags,
       instead of the input's default ~20ch width */
    flex: 1;
    width: 0;
    padding: 0;
    font: 12px var(--mono);
    text-align: right;
    color: var(--text);
    background: none;
    border: none;
    outline: none;
    appearance: textfield;
  }
  input::-webkit-inner-spin-button,
  input::-webkit-outer-spin-button {
    appearance: none;
    margin: 0;
  }

  /* Unit picker: looks like the plain unit tag, with a small caret */
  .unit {
    position: relative;
    flex: none;
    display: flex;
    align-items: center;
  }
  select {
    padding: 2px 12px 2px 2px;
    font: 11px var(--mono);
    color: var(--text-dim);
    background: none;
    border: none;
    border-radius: 4px;
    outline: none;
    appearance: none;
    cursor: pointer;
  }
  select:hover,
  select:focus-visible {
    color: var(--text);
    background: var(--border);
  }
  /* The open option list is drawn by the browser: give it the panel's colours
     instead of inheriting the dim tag colour on a light default background. */
  select {
    color-scheme: dark;
  }
  option {
    color: var(--text);
    background: var(--panel-solid);
  }

  .caret {
    position: absolute;
    right: 2px;
    font-size: 8px;
    color: var(--text-dim);
    pointer-events: none;
  }
</style>
