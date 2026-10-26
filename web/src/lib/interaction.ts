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
  return roundTo(v, gridStep(cam.scale) / 10)
}

// Velocities set with the arrow: the arrow is `arrowTime` seconds of travel,
// so a tenth of a grid step of arrow is that much less velocity. Rounded to a
// power of ten so the fields stay readable (1.2 km/s, not 1.157407 km/s).
function snapVel(cam: Camera, v: number, arrowTime: number): number {
  return roundTo(v, 10 ** Math.floor(Math.log10(gridStep(cam.scale) / 10 / arrowTime)))
}

function roundTo(v: number, q: number): number {
  return Number((Math.round(v / q) * q).toPrecision(12))
}

// How the canvas talks to the follow-a-body feature.
export interface Follow {
  bodyAt(sx: number, sy: number): number | null // id of the body under a screen point
  toggle(id: number): void
  stop(): void
}

// A press that moves less than this (px) is a click, not a pan.
const CLICK_SLOP_PX = 4

// Wheel zoom, drag-to-pan, clicking a body to follow it, and placing /
// dragging the draft body. `arrowTime` gives the current length of time the
// velocity arrow stands for (see draft.ts). Returns a function that removes
// all listeners.
export function attachInteraction(
  canvas: HTMLCanvasElement,
  size: Size,
  cam: Camera,
  drafts: DraftState,
  follow: Follow,
  arrowTime: () => number,
) {
  // What the current pointer drag is doing.
  let drag: 'pan' | 'position' | 'velocity' | null = null
  let grabDx = 0, grabDy = 0 // keeps the body from jumping to the cursor when grabbed off-centre
  let panMoved = 0 // px dragged so far in the current pan

  const toWorld = (e: PointerEvent | WheelEvent) => screenToWorld(cam, size.w, size.h, e.offsetX, e.offsetY)
  const hit = (d: Draft, e: PointerEvent) => hitDraft(cam, size.w, size.h, d, arrowTime(), e.offsetX, e.offsetY)

  const setPos = (d: Draft, wx: number, wy: number) => {
    d.x = snap(cam, wx + grabDx)
    d.y = snap(cam, wy + grabDy)
  }
  // The derived |v| and θ are rounded too (θ to 0.1°), so the fields stay short.
  // That leaves them a hair off the exact polar form of (vx, vy), which is
  // below what the mouse can place anyway. Typed values never go through here.
  const setVel = (d: Draft, wx: number, wy: number) => {
    const t = arrowTime()
    setVelocityXY(d, snapVel(cam, (wx - d.x) / t, t), snapVel(cam, (wy - d.y) / t, t))
    d.speed = snapVel(cam, d.speed, t)
    d.angle = Number((Math.round(d.angle * 10) / 10).toFixed(1))
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
    panMoved = 0
  }

  const onMove = (e: PointerEvent) => {
    const [wx, wy] = toWorld(e)
    const d = drafts.draft
    const mode = drafts.mode

    if (drag === 'pan') {
      panMoved += Math.abs(e.movementX) + Math.abs(e.movementY)
      if (panMoved > CLICK_SLOP_PX) follow.stop() // a real drag: let go of the followed body
      pan(cam, e.movementX, e.movementY)
    }
    else if (d && (drag === 'position' || (!drag && mode === 'placing'))) setPos(d, wx, wy)
    else if (d && (drag === 'velocity' || (!drag && mode === 'aiming'))) setVel(d, wx, wy)

    // cursor hint
    if (!d) canvas.style.cursor = !drag && follow.bodyAt(e.offsetX, e.offsetY) !== null ? 'pointer' : ''
    else if (mode !== 'editing') canvas.style.cursor = 'crosshair'
    else if (!drag) {
      const target = hit(d, e)
      canvas.style.cursor = target === 'position' ? 'move' : target === 'velocity' ? 'crosshair' : ''
    }
  }

  const onUp = (e: PointerEvent) => {
    // a press that didn't really move is a click: follow (or unfollow) the body under it
    if (e.type === 'pointerup' && drag === 'pan' && panMoved <= CLICK_SLOP_PX) {
      const id = follow.bodyAt(e.offsetX, e.offsetY)
      if (id !== null) follow.toggle(id)
    }
    drag = null
  }

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
