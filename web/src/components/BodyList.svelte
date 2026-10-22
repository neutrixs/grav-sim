<script lang="ts">
  import { sim } from '../state/sim.svelte'

  const fmt = (v: number) => v.toFixed(3)
</script>

<h2 class="section-title">Bodies <span class="count">{sim.bodies.length}</span></h2>

{#if sim.bodies.length === 0}
  <p class="empty">None yet</p>
{:else}
  <table>
    <thead>
      <tr><th>id</th><th>x</th><th>y</th></tr>
    </thead>
    <tbody>
      {#each sim.bodies as b (b.id)}
        <tr><td>#{b.id}</td><td>{fmt(b.x)}</td><td>{fmt(b.y)}</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  .count {
    margin-left: 4px;
    color: var(--text);
  }
  .empty {
    margin: 0;
    font-size: 12px;
    color: var(--text-dim);
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font: 12px var(--mono);
  }
  th,
  td {
    text-align: right;
  }
  th {
    font-weight: normal;
    color: var(--text-dim);
    padding-bottom: 4px;
  }
  td {
    padding: 2px 0;
  }
  th:first-child,
  td:first-child {
    text-align: left;
    color: var(--text-dim);
  }
</style>
