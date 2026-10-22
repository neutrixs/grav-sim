<script lang="ts">
  import { view } from '../state/view.svelte'
</script>

<div>
  <span class="tick"></span>
  {view.gridLabel}
</div>

<style>
  div {
    position: fixed;
    left: var(--edge);
    bottom: var(--edge);
    display: flex;
    align-items: center;
    gap: 8px;
    font: 12px/1 var(--mono);
    color: var(--text-dim);
    pointer-events: none;
    user-select: none;
  }

  .tick {
    width: 10px;
    height: 10px;
    border: 1px solid var(--text-dim);
  }
</style>
