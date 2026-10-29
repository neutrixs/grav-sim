import type { Sim } from '../../pkg'
import { type Camera, worldToScreen } from './camera'

// Plain-JS copy of a body, so we don't hold on to Rust handles between frames.
export interface BodyData {
  id: number
  x: number
  y: number
  vx: number
  vy: number
  mass: number
  radius: number
}

// Pull the current state of every body out of Rust. Each Body is a
// handle to memory on the Rust side, so it is copied and freed right away.
export function snapshot(rs: Sim): BodyData[] {
  return rs.retrieve().map((b) => {
    const d = { id: b.id, x: b.pos_x, y: b.pos_y, vx: b.v_x, vy: b.v_y, mass: b.mass, radius: b.radius }
    b.free()
    return d
  })
}

// Bodies shrinking below this on screen are still drawn at this size.
export const MIN_RADIUS_PX = 3
// Extra click margin around a body, so tiny ones are still easy to hit.
const HIT_SLACK_PX = 4

// The body drawn under screen point (sx, sy), or null. Later bodies are
// drawn on top, so they win when several overlap.
export function bodyAt(cam: Camera, w: number, h: number, bodies: BodyData[], sx: number, sy: number): BodyData | null {
  for (let i = bodies.length - 1; i >= 0; i--) {
    const b = bodies[i]
    const [bx, by] = worldToScreen(cam, w, h, b.x, b.y)
    const r = Math.max(MIN_RADIUS_PX, b.radius * cam.scale) + HIT_SLACK_PX
    if ((sx - bx) ** 2 + (sy - by) ** 2 <= r * r) return b
  }
  return null
}

export function drawBodies(
  ctx: CanvasRenderingContext2D,
  cam: Camera,
  w: number,
  h: number,
  bodies: BodyData[],
  fill: string,
  label: string,
) {
  ctx.font = '11px ui-monospace, monospace'
  ctx.textBaseline = 'middle'

  for (const b of bodies) {
    const [sx, sy] = worldToScreen(cam, w, h, b.x, b.y)
    const r = Math.max(MIN_RADIUS_PX, b.radius * cam.scale)

    ctx.fillStyle = fill
    ctx.beginPath()
    ctx.arc(sx, sy, r, 0, Math.PI * 2)
    ctx.fill()

    ctx.fillStyle = label
    ctx.fillText(`#${b.id}`, sx + r + 6, sy)
  }
}
