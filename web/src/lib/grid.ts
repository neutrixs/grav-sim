import { type Camera, screenToWorld } from './camera'

// Thinnest lines are never closer than this on screen.
const MIN_GAP_PX = 8
const MAX_ALPHA = 0.35

// A line's weight depends only on its on-screen gap, so it changes continuously
// while zooming: 0 at MIN_GAP_PX (invisible), 1 at 100× that (full thickness).
function weight(gapPx: number): number {
  return Math.min(1, Math.max(0, Math.log10(gapPx / MIN_GAP_PX) / 2))
}

// Spacing of the thinnest visible grid level: the smallest power of 10 metres
// whose gap on screen is at least MIN_GAP_PX.
export function gridStep(scale: number): number {
  return 10 ** Math.ceil(Math.log10(MIN_GAP_PX / scale))
}

export function drawGrid(ctx: CanvasRenderingContext2D, cam: Camera, w: number, h: number, color: string) {
  const step = gridStep(cam.scale)
  const [left, top] = screenToWorld(cam, w, h, 0, 0)
  const [right, bottom] = screenToWorld(cam, w, h, w, h)
  const x0 = Math.floor(left / step), x1 = Math.ceil(right / step)
  const y0 = Math.floor(bottom / step), y1 = Math.ceil(top / step)

  ctx.strokeStyle = color

  // Three levels: step, 10·step, 100·step. Each line is drawn once, at the
  // coarsest level its index belongs to (checked on the integer index k,
  // never on the float position).
  for (let level = 0; level < 3; level++) {
    const wgt = weight(step * 10 ** level * cam.scale)
    if (wgt === 0) continue

    ctx.globalAlpha = wgt * MAX_ALPHA
    ctx.lineWidth = 1 + wgt
    ctx.beginPath()

    const belongs = (k: number) => {
      const a = Math.abs(k)
      if (level === 2) return a % 100 === 0
      if (level === 1) return a % 10 === 0 && a % 100 !== 0
      return a % 10 !== 0
    }

    for (let k = x0; k <= x1; k++) {
      if (!belongs(k)) continue
      const sx = Math.round((k * step - cam.x) * cam.scale + w / 2) + 0.5
      ctx.moveTo(sx, 0)
      ctx.lineTo(sx, h)
    }
    for (let k = y0; k <= y1; k++) {
      if (!belongs(k)) continue
      const sy = Math.round(h / 2 - (k * step - cam.y) * cam.scale) + 0.5
      ctx.moveTo(0, sy)
      ctx.lineTo(w, sy)
    }
    ctx.stroke()
  }

  ctx.globalAlpha = 1
}

// "1 m", "10 km", "1 000 000 km", "1 cm", ...
export function formatLength(m: number): string {
  const nf = (v: number) => v.toLocaleString('en-US', { maximumFractionDigits: 3 }).replace(/,/g, ' ')
  if (m >= 1e3) return `${nf(m / 1e3)} km`
  if (m >= 1) return `${nf(m)} m`
  if (m >= 1e-2) return `${nf(m * 1e2)} cm`
  return `${nf(m * 1e3)} mm`
}
