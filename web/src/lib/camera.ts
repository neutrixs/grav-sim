// World space is in metres with +y up; screen space is CSS pixels with +y down.
export interface Camera {
  x: number // world point at the centre of the screen
  y: number
  scale: number // pixels per metre
}

export const MIN_SCALE = 1e-12
export const MAX_SCALE = 1e6

export function worldToScreen(cam: Camera, w: number, h: number, wx: number, wy: number): [number, number] {
  return [(wx - cam.x) * cam.scale + w / 2, h / 2 - (wy - cam.y) * cam.scale]
}

export function screenToWorld(cam: Camera, w: number, h: number, sx: number, sy: number): [number, number] {
  return [(sx - w / 2) / cam.scale + cam.x, (h / 2 - sy) / cam.scale + cam.y]
}

// Zoom by `factor`, keeping the world point under (sx, sy) fixed on screen.
export function zoomAt(cam: Camera, w: number, h: number, sx: number, sy: number, factor: number) {
  const [wx, wy] = screenToWorld(cam, w, h, sx, sy)
  cam.scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, cam.scale * factor))
  cam.x = wx - (sx - w / 2) / cam.scale
  cam.y = wy - (h / 2 - sy) / cam.scale
}

export function pan(cam: Camera, dxPx: number, dyPx: number) {
  cam.x -= dxPx / cam.scale
  cam.y += dyPx / cam.scale
}
