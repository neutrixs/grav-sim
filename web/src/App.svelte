<script lang="ts">
  import { onMount } from 'svelte'
  import { type Camera, zoomAt, pan } from './lib/camera'
  import { drawGrid, gridStep, formatLength } from './lib/grid'

  let canvas: HTMLCanvasElement
  const cam: Camera = { x: 0, y: 0, scale: 50 }
  let gridLabel = $state('')

  onMount(() => {
    const ctx = canvas.getContext('2d')!
    let w = 0, h = 0

    // Keep the canvas's pixel buffer matched to its CSS size × devicePixelRatio,
    // then draw in CSS pixels.
    const resize = () => {
      const dpr = window.devicePixelRatio || 1
      w = canvas.clientWidth
      h = canvas.clientHeight
      canvas.width = Math.round(w * dpr)
      canvas.height = Math.round(h * dpr)
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    }
    const ro = new ResizeObserver(resize)
    ro.observe(canvas)
    resize()

    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      const px = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY // line-mode wheels
      zoomAt(cam, w, h, e.offsetX, e.offsetY, Math.exp(-px * 0.0015))
    }
    canvas.addEventListener('wheel', onWheel, { passive: false })

    let dragging = false
    const onDown = (e: PointerEvent) => {
      dragging = true
      canvas.setPointerCapture(e.pointerId)
    }
    const onMove = (e: PointerEvent) => {
      if (dragging) pan(cam, e.movementX, e.movementY)
    }
    const onUp = () => (dragging = false)
    canvas.addEventListener('pointerdown', onDown)
    canvas.addEventListener('pointermove', onMove)
    canvas.addEventListener('pointerup', onUp)
    canvas.addEventListener('pointercancel', onUp)

    const gridColor = getComputedStyle(canvas).getPropertyValue('--grid').trim()

    let raf = 0
    const frame = () => {
      ctx.clearRect(0, 0, w, h)
      drawGrid(ctx, cam, w, h, gridColor)
      gridLabel = formatLength(gridStep(cam.scale) * 10)
      raf = requestAnimationFrame(frame)
    }
    raf = requestAnimationFrame(frame)

    return () => {
      cancelAnimationFrame(raf)
      ro.disconnect()
      canvas.removeEventListener('wheel', onWheel)
    }
  })
</script>

<canvas bind:this={canvas}></canvas>

<div class="scale">
  <span class="tick"></span>
  {gridLabel}
</div>

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

  .scale {
    position: fixed;
    left: 16px;
    bottom: 16px;
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
