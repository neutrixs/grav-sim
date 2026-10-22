import { type Camera, pan, screenToWorld, zoomAt } from './camera'
import { gridStep } from './grid'
import { type Draft, hitDraft, setVelocityXY } from './draft'
import type { DraftState } from '../state/draft.svelte'

// Canvas size in CSS pixels, kept up to date by the owner of the canvas.
export interface Size {
  w: number
  h: number
}

// Values set with the cursor are rounded to a tenth of the finest visible
// grid step, so the fields show e.g. 12.3 instead of 12.30481920394.
function snap(cam: Camera, v: number): number {
  const q = gridStep(cam.scale) / 10
  return Number((Math.round(v / q) * q).toPrecision(12))
}

// Wheel zoom, drag-to-pan, and placing / dragging the draft body.
// Returns a function that removes all listeners.
export function attachInteraction(canvas: HTMLCanvasElement, size: Size, cam: Camera, drafts: DraftState) {
  // What the current pointer drag is doing.
  let drag: 'pan' | 'position' | 'velocity' | null = null
  let grabDx = 0, grabDy = 0 // keeps the body from jumping to the cursor when grabbed off-centre

  const toWorld = (e: PointerEvent | WheelEvent) => screenToWorld(cam, size.w, size.h, e.offsetX, e.offsetY)
  const hit = (d: Draft, e: PointerEvent) => hitDraft(cam, size.w, size.h, d, e.offsetX, e.offsetY)

  const setPos = (d: Draft, wx: number, wy: number) => {
    d.x = snap(cam, wx + grabDx)
    d.y = snap(cam, wy + grabDy)
  }
  const setVel = (d: Draft, wx: number, wy: number) => {
    setVelocityXY(d, snap(cam, wx - d.x), snap(cam, wy - d.y))
  }

  const onWheel = (e: WheelEvent) => {
    e.preventDefault()
    const px = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY // line-mode wheels
    zoomAt(cam, size.w, size.h, e.offsetX, e.offsetY, Math.exp(-px * 0.0015))
  }

  const onDown = (e: PointerEvent) => {
    if (e.button !== 0) return
    canvas.setPointerCapture(e.pointerId)
    const [wx, wy] = toWorld(e)
    const d = drafts.draft
    grabDx = grabDy = 0

    if (d && drafts.mode === 'placing') {
      setPos(d, wx, wy)
      drafts.mode = 'aiming'
      return
    }
    if (d && drafts.mode === 'aiming') {
      setVel(d, wx, wy)
      drafts.mode = 'editing'
      return
    }
    if (d && drafts.mode === 'editing') {
      const target = hit(d, e)
      if (target === 'position') {
        grabDx = d.x - wx
        grabDy = d.y - wy
      }
      if (target) {
        drag = target
        return
      }
    }
    drag = 'pan'
  }

  const onMove = (e: PointerEvent) => {
    const [wx, wy] = toWorld(e)
    const d = drafts.draft
    const mode = drafts.mode

    if (drag === 'pan') pan(cam, e.movementX, e.movementY)
    else if (d && (drag === 'position' || (!drag && mode === 'placing'))) setPos(d, wx, wy)
    else if (d && (drag === 'velocity' || (!drag && mode === 'aiming'))) setVel(d, wx, wy)

    // cursor hint
    if (!d) canvas.style.cursor = ''
    else if (mode !== 'editing') canvas.style.cursor = 'crosshair'
    else if (!drag) {
      const target = hit(d, e)
      canvas.style.cursor = target === 'position' ? 'move' : target === 'velocity' ? 'crosshair' : ''
    }
  }

  const onUp = () => (drag = null)

  canvas.addEventListener('wheel', onWheel, { passive: false })
  canvas.addEventListener('pointerdown', onDown)
  canvas.addEventListener('pointermove', onMove)
  canvas.addEventListener('pointerup', onUp)
  canvas.addEventListener('pointercancel', onUp)

  return () => {
    canvas.removeEventListener('wheel', onWheel)
    canvas.removeEventListener('pointerdown', onDown)
    canvas.removeEventListener('pointermove', onMove)
    canvas.removeEventListener('pointerup', onUp)
    canvas.removeEventListener('pointercancel', onUp)
  }
}
