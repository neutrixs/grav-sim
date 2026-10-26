<script lang="ts">
  import { onMount } from 'svelte'
  import { drawGrid, gridStep, formatLength } from '../lib/grid'
  import { bodyAt, drawBodies } from '../lib/bodies'
  import { drawDraft } from '../lib/draft'
  import { type Size, attachInteraction } from '../lib/interaction'
  import { sim } from '../state/sim.svelte'
  import { drafts } from '../state/draft.svelte'
  import { view } from '../state/view.svelte'
  import { unitOf } from '../state/units.svelte'

  let canvas: HTMLCanvasElement

  onMount(() => {
    const ctx = canvas.getContext('2d')!
    const size: Size = { w: 0, h: 0 }
    const cam = view.cam

    // Keep the canvas's pixel buffer matched to its CSS size × devicePixelRatio,
    // then draw in CSS pixels.
    const resize = () => {
      const dpr = window.devicePixelRatio || 1
      size.w = canvas.clientWidth
      size.h = canvas.clientHeight
      canvas.width = Math.round(size.w * dpr)
      canvas.height = Math.round(size.h * dpr)
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    }
    const ro = new ResizeObserver(resize)
    ro.observe(canvas)
    resize()

    const detach = attachInteraction(canvas, size, cam, drafts, {
      bodyAt: (sx, sy) => bodyAt(cam, size.w, size.h, sim.bodies, sx, sy)?.id ?? null,
      toggle: (id) => view.toggleFollow(id),
      stop: () => (view.following = null),
    }, () => sim.speed)

    // Canvas can't use CSS variables directly, so read the tokens once.
    const css = getComputedStyle(canvas)
    const color = (name: string) => css.getPropertyValue(name).trim()
    const colors = {
      grid: color('--grid'),
      body: color('--body'),
      label: color('--text-dim'),
      draft: color('--accent'),
      bg: color('--bg'),
    }

    sim.sync()

    let raf = 0
    const frame = (now: number) => {
      if (sim.running) sim.tick(now)

      // Keep the followed body in the centre (zoom still works around it).
      if (view.following !== null) {
        const b = sim.bodies.find((b) => b.id === view.following)
        if (b) {
          cam.x = b.x
          cam.y = b.y
        } else view.following = null
      }

      const { w, h } = size
      ctx.clearRect(0, 0, w, h)
      drawGrid(ctx, cam, w, h, colors.grid)
      drawBodies(ctx, cam, w, h, sim.bodies, colors.body, colors.label)
      if (drafts.draft) drawDraft(ctx, cam, w, h, drafts.draft, drafts.mode !== 'placing', colors.draft, colors.bg, unitOf('velocity'), sim.speed)
      view.gridLabel = formatLength(gridStep(cam.scale) * 10)

      raf = requestAnimationFrame(frame)
    }
    raf = requestAnimationFrame(frame)

    return () => {
      cancelAnimationFrame(raf)
      ro.disconnect()
      detach()
    }
  })
</script>

<canvas bind:this={canvas}></canvas>

<style>
  canvas {
    position: fixed;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
    touch-action: none;
    cursor: grab;
  }
  canvas:active {
    cursor: grabbing;
  }
</style>
