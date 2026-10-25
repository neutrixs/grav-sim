<script lang="ts">
  import { sim } from '../state/sim.svelte'
  import { formatDistance } from '../lib/units'
</script>

<h2 class="section-title">Bodies <span class="count">{sim.bodies.length}</span></h2>

{#if sim.bodies.length === 0}
  <p class="empty">None yet</p>
{:else}
  <table>
    <colgroup><col class="id" /><col /><col /></colgroup>
    <thead>
      <tr><th>id</th><th>x</th><th>y</th></tr>
    </thead>
    <tbody>
      {#each sim.bodies as b (b.id)}
        <tr><td>#{b.id}</td><td>{formatDistance(b.x)}</td><td>{formatDistance(b.y)}</td></tr>
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
    /* fixed column widths, so changing numbers don't shift the columns */
    table-layout: fixed;
  }
  col.id {
    width: 28px;
  }
  th,
  td {
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  /* keep the x and y columns from running into each other */
  th + th,
  td + td {
    padding-left: 8px;
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
