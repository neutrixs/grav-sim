import type { Camera } from '../lib/camera'

class View {
  // Not reactive on purpose: it's read and written every frame by the canvas,
  // and nothing in the markup depends on it directly.
  cam: Camera = { x: 0, y: 0, scale: 50 }

  // Spacing of the thick grid lines, e.g. "10 m". Updated by the renderer.
  gridLabel = $state('')
}

export const view = new View()
