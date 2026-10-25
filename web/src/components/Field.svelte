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
  const shown = $derived(numText(toDisplay(value, unit)))

  // Plain digits up to 1e12, e-notation beyond ("1e19" instead of twenty
  // digits). Still exact, and still parses back when edited.
  function numText(v: number): string {
    return (Math.abs(v) >= 1e12 ? v.toExponential() : String(v)).replace('e+', 'e')
  }

  // While focused, the input holds free text that isn't parsed, so half-typed
  // numbers like "0.00" or "1e" are left alone. It's only parsed and applied
  // on Enter or blur; anything invalid snaps back to the last value.
  let editing = $state(false)
  let text = $state('')

  function commit() {
    const raw = text.trim()
    const n = Number(raw)
    if (raw !== '' && Number.isFinite(n) && (min === undefined || n >= min)) {
      value = fromDisplay(n, unit)
    }
    // the new value, or the old one if invalid / rejected by the setter
    text = numText(toDisplay(value, unit))
  }

  function onkeydown(e: KeyboardEvent) {
    // no preventDefault: Enter still submits the form, after the commit
    if (e.key === 'Enter') commit()
  }
</script>

<label class="field">
  <span class="tag">{label}</span>
  <input
    type="text"
    inputmode="decimal"
    autocomplete="off"
    spellcheck="false"
    value={editing ? text : shown}
    oninput={(e) => (text = e.currentTarget.value)}
    onfocus={() => {
      text = shown
      editing = true
    }}
    onblur={() => {
      commit()
      editing = false
    }}
    {onkeydown}
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
