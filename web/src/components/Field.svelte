<script lang="ts">
  // Number input laid out as [label | value | unit] in one box.
  interface Props {
    label: string
    unit: string
    value: number
    min?: number
  }
  let { label, unit, value = $bindable(), min }: Props = $props()
</script>

<label class="field">
  <span class="tag">{label}</span>
  <input type="number" step="any" {min} bind:value />
  <span class="tag">{unit}</span>
</label>

<style>
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 8px;
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
</style>
