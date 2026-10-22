<script lang="ts">
  import DraftForm from './DraftForm.svelte'
  import BodyList from './BodyList.svelte'
  import { drafts } from '../state/draft.svelte'
  import { view } from '../state/view.svelte'
</script>

<aside class="surface">
  <section>
    {#if drafts.draft}
      <DraftForm />
    {:else}
      <button class="btn wide" onclick={() => drafts.start(view.cam.x, view.cam.y)}>Place body</button>
    {/if}
  </section>

  <section class="list">
    <BodyList />
  </section>
</aside>

<style>
  aside {
    position: fixed;
    top: var(--edge);
    left: var(--edge);
    width: 264px;
    max-height: calc(100vh - 72px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  section {
    padding: 14px;
  }
  section + section {
    border-top: 1px solid var(--border);
  }

  .list {
    overflow-y: auto;
    min-height: 0;
  }

  .wide {
    width: 100%;
  }
</style>
