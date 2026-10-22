import { type Camera, worldToScreen } from './camera'
import { MIN_RADIUS_PX } from './bodies'

// A body being set up in the UI. Lives only on the JS side until it's added.
export interface Draft {
  x: number
  y: number
  vx: number
  vy: number
  // Polar form of (vx, vy), kept in sync by the setters below. The angle is
  // stored rather than only derived, so it survives the speed going through 0.
  speed: number // m/s
  angle: number // degrees, 0 = +x, counter-clockwise
  mass: number
  radius: number
}

// Trim float noise (e.g. 4.999999999999999 → 5) so the fields stay readable.
const clean = (v: number) => Number(v.toPrecision(12))

// vx/vy changed → recompute speed and angle.
export function setVelocityXY(d: Draft, vx: number, vy: number) {
  d.vx = vx
  d.vy = vy
  if (!Number.isFinite(vx) || !Number.isFinite(vy)) return
  d.speed = clean(Math.hypot(vx, vy))
  if (d.speed > 0) d.angle = clean((((Math.atan2(vy, vx) * 180) / Math.PI) % 360 + 360) % 360)
}

// Speed changed → keep the angle.
export function setSpeed(d: Draft, speed: number) {
  if (!Number.isFinite(speed) || speed < 0) return
  d.speed = speed
  polarToXY(d)
}

// Angle changed → keep the speed.
export function setAngle(d: Draft, angle: number) {
  if (!Number.isFinite(angle)) return
  d.angle = angle
  polarToXY(d)
}

function polarToXY(d: Draft) {
  const rad = (d.angle * Math.PI) / 180
  // cos(90°) is 6e-17 in floats, not 0: snap tiny components to exact 0
  const tidy = (v: number) => (Math.abs(v) < d.speed * 1e-12 ? 0 : clean(v))
  d.vx = tidy(d.speed * Math.cos(rad))
  d.vy = tidy(d.speed * Math.sin(rad))
}

// placing: ghost follows the cursor, click drops it
// aiming:  velocity follows the cursor, click sets it
// editing: drag the body / arrow tip, or type in the fields
export type DraftMode = 'placing' | 'aiming' | 'editing'

// How close (px) the pointer must be to grab the arrow tip or a small body.
const GRAB_PX = 10
const HEAD_PX = 10

// The arrow tip sits where the body would be after 1 s, so its length in
// metres equals the speed in m/s.
function screenPoints(cam: Camera, w: number, h: number, d: Draft) {
  const [cx, cy] = worldToScreen(cam, w, h, d.x, d.y)
  const [tx, ty] = worldToScreen(cam, w, h, d.x + d.vx, d.y + d.vy)
  const r = Math.max(MIN_RADIUS_PX, d.radius * cam.scale)
  return { cx, cy, tx, ty, r }
}

// What's under the pointer. The arrow tip wins over the body, so a zero
// velocity (tip at the centre) can still be pulled out.
export function hitDraft(cam: Camera, w: number, h: number, d: Draft, sx: number, sy: number) {
  const p = screenPoints(cam, w, h, d)
  if (Math.hypot(sx - p.tx, sy - p.ty) <= GRAB_PX) return 'velocity'
  if (Math.hypot(sx - p.cx, sy - p.cy) <= Math.max(p.r, GRAB_PX)) return 'position'
  return null
}

export function drawDraft(
  ctx: CanvasRenderingContext2D,
  cam: Camera,
  w: number,
  h: number,
  d: Draft,
  showArrow: boolean,
  color: string,
  bg: string,
) {
  const { cx, cy, tx, ty, r } = screenPoints(cam, w, h, d)
  ctx.save()
  ctx.strokeStyle = color
  ctx.fillStyle = color
  ctx.lineWidth = 1.5

  // ghost body
  ctx.beginPath()
  ctx.arc(cx, cy, r, 0, Math.PI * 2)
  ctx.globalAlpha = 0.2
  ctx.fill()
  ctx.globalAlpha = 1
  ctx.setLineDash([4, 4])
  ctx.stroke()
  ctx.setLineDash([])

  if (showArrow) {
    const len = Math.hypot(tx - cx, ty - cy)
    if (len > 1) {
      // shaft, stopping at the base of the head
      const ux = (tx - cx) / len, uy = (ty - cy) / len
      const head = Math.min(HEAD_PX, len)
      ctx.beginPath()
      ctx.moveTo(cx, cy)
      ctx.lineTo(tx - ux * head, ty - uy * head)
      ctx.stroke()

      // head
      ctx.beginPath()
      ctx.moveTo(tx, ty)
      ctx.lineTo(tx - ux * head - uy * head * 0.5, ty - uy * head + ux * head * 0.5)
      ctx.lineTo(tx - ux * head + uy * head * 0.5, ty - uy * head - ux * head * 0.5)
      ctx.closePath()
      ctx.fill()

      ctx.font = '11px ui-monospace, monospace'
      ctx.textBaseline = 'middle'
      ctx.fillText(`${formatSpeed(Math.hypot(d.vx, d.vy))} m/s`, tx + ux * 10 + 4, ty + uy * 10)
    }

    // grab handle at the tip
    ctx.beginPath()
    ctx.arc(tx, ty, 4, 0, Math.PI * 2)
    ctx.fillStyle = bg
    ctx.fill()
    ctx.stroke()
  }

  ctx.restore()
}

function formatSpeed(v: number): string {
  return Number(v.toPrecision(4)).toString()
}
