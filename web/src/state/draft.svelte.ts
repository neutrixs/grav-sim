import type { Draft, DraftMode } from '../lib/draft'
import { sim } from './sim.svelte'

// The body being set up on the canvas / in the panel. Only sent to Rust on confirm.
export class DraftState {
  draft: Draft | null = $state(null)
  mode: DraftMode = $state('placing')
  error = $state('')

  // New drafts start with the mass/radius of the last added body.
  #lastMass = 1
  #lastRadius = 1

  start(x: number, y: number) {
    this.draft = { x, y, vx: 0, vy: 0, speed: 0, angle: 0, mass: this.#lastMass, radius: this.#lastRadius }
    this.mode = 'placing'
    this.error = ''
  }

  cancel() {
    this.draft = null
    this.error = ''
  }

  confirm() {
    const d = this.draft
    if (!d) return
    if (![d.x, d.y, d.vx, d.vy, d.mass, d.radius].every(Number.isFinite)) {
      this.error = 'All fields must be numbers'
      return
    }
    try {
      sim.addBody(d)
      this.#lastMass = d.mass
      this.#lastRadius = d.radius
      this.cancel()
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err)
    }
  }
}

export const drafts = new DraftState()
