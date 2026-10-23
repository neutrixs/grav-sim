<script lang="ts">
  import Field from './Field.svelte'
  import { type StepMode, sim } from '../state/sim.svelte'
  import { formatClock, formatDuration } from '../lib/time'

  const modes: { id: StepMode; label: string }[] = [
    { id: 'fixed', label: 'Fixed' },
    { id: 'dynamic', label: 'Dynamic' },
  ]

  // Physics steps per real second at the current settings.
  const stepsPerSec = $derived(Math.round(sim.speed / sim.dt))
</script>

<aside class="surface">
  <header>
    <button class="btn play" onclick={() => sim.toggle()} aria-label={sim.running ? 'Pause' : 'Play'}>
      {sim.running ? '❚❚' : '▶'}
    </button>
    <div class="status">
      <span class="clock">t = {formatClock(sim.time)}</span>
      {#if sim.running && sim.slowdown > 1.05}
        <span class="lag">
          {Number.isFinite(sim.slowdown) ? `${sim.slowdown.toFixed(1)}× slower` : 'stalled'}
        </span>
      {/if}
    </div>
  </header>

  <section>
    <h2 class="section-title">Time</h2>

    <Field label="speed" units="speed" bind:value={() => sim.speed, (v) => sim.setSpeed(v)} />

    <div class="label-row">
      <span>Step</span>
      <div class="segmented" role="radiogroup" aria-label="Step mode">
        {#each modes as m (m.id)}
          <button
            role="radio"
            aria-checked={sim.stepMode === m.id}
            class:active={sim.stepMode === m.id}
            onclick={() => sim.setStepMode(m.id)}>{m.label}</button
          >
        {/each}
      </div>
    </div>

    {#if sim.stepMode === 'fixed'}
      <Field label="dt" units="dt" bind:value={() => sim.fixedDt, (v) => sim.setFixedDt(v)} />
    {:else}
      <Field label="dt =" unit="× speed" bind:value={() => sim.dtFactor, (v) => sim.setDtFactor(v)} />
    {/if}

    <p class="readout">
      dt = {formatDuration(sim.dt)} · {stepsPerSec.toLocaleString('en-US')} steps/s
    </p>
  </section>
</aside>

<style>
  aside {
    position: fixed;
    top: var(--edge);
    right: var(--edge);
    width: 264px;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
  }
  .play {
    width: 32px;
    height: 32px;
    padding: 0;
    font-size: 12px;
  }
  .status {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .clock {
    font: 13px var(--mono);
  }
  .lag {
    font: 11px var(--mono);
    color: var(--warn);
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px;
    border-top: 1px solid var(--border);
  }
  section .section-title {
    margin-bottom: 2px;
  }

  .label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 4px;
    font-size: 11px;
    color: var(--text-dim);
  }

  /* Two-option toggle */
  .segmented {
    display: flex;
    padding: 2px;
    background: var(--input);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .segmented button {
    padding: 3px 10px;
    font: inherit;
    font-size: 11px;
    color: var(--text-dim);
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  .segmented button:hover {
    color: var(--text);
  }
  .segmented button.active {
    color: var(--bg);
    background: var(--accent);
  }

  .readout {
    margin: 0;
    font: 11px var(--mono);
    color: var(--text-dim);
  }
</style>
