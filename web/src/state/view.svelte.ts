import type { Camera } from '../lib/camera'

class View {
  // Not reactive on purpose: it's read and written every frame by the canvas,
  // and nothing in the markup depends on it directly.
  cam: Camera = { x: 0, y: 0, scale: 50 }

  // Spacing of the thick grid lines, e.g. "10 m". Updated by the renderer.
  gridLabel = $state('')

  // Id of the body the camera keeps centred, if any. Panning stops it.
  following: number | null = $state(null)

  // While following: grid lines pass through the followed body (true) or
  // stay fixed to the world origin (false).
  gridFollows = $state(true)

  toggleFollow(id: number) {
    this.following = this.following === id ? null : id
  }
}

export const view = new View()
