<script lang="ts">
  import { sim } from '../state/sim.svelte'
  import { view } from '../state/view.svelte'
  import { formatDistance } from '../lib/units'
</script>

<h2 class="section-title">Bodies <span class="count">{sim.bodies.length}</span></h2>

{#if view.following !== null}
  <label class="grid-toggle">
    <input type="checkbox" bind:checked={view.gridFollows} />
    Grid moves with #{view.following}
  </label>
{/if}

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
        {@const followed = view.following === b.id}
        <tr class:followed>
          <td>
            <button
              class="follow"
              aria-pressed={followed}
              title={followed ? 'Stop following' : 'Keep this body in the centre'}
              onclick={() => view.toggleFollow(b.id)}>#{b.id}</button
            >
          </td>
          <td>{formatDistance(b.x)}</td>
          <td>{formatDistance(b.y)}</td>
        </tr>
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

  .grid-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: -2px 0 8px;
    font-size: 12px;
    color: var(--text-dim);
    cursor: pointer;
    user-select: none;
  }
  .grid-toggle input {
    margin: 0;
    accent-color: var(--accent);
    cursor: pointer;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font: 12px var(--mono);
    /* fixed column widths, so changing numbers don't shift the columns */
    table-layout: fixed;
  }
  col.id {
    width: 36px;
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

  /* The id doubles as the follow toggle */
  .follow {
    padding: 0 4px;
    font: inherit;
    color: inherit;
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  .follow:hover {
    color: var(--text);
    background: var(--border);
  }
  .followed td {
    color: var(--accent);
  }
  .followed .follow {
    color: var(--bg);
    background: var(--accent);
  }
</style>
