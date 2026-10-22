<script lang="ts">
  import { sim } from '../state/sim.svelte'
</script>

<div class="surface">
  <button class="btn" onclick={() => sim.toggle()} aria-label={sim.running ? 'Pause' : 'Play'}>
    {sim.running ? '❚❚' : '▶'}
  </button>
  <span class="clock">t = {sim.time.toFixed(3)} s</span>
</div>

<style>
  div {
    position: fixed;
    top: var(--edge);
    right: var(--edge);
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 14px 6px 6px;
  }

  button {
    width: 32px;
    height: 32px;
    padding: 0;
    font-size: 12px;
  }

  .clock {
    min-width: 12ch;
    font: 13px var(--mono);
  }
</style>
