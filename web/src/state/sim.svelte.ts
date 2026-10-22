import { SimulationManager } from '../../pkg'
import { type BodyData, snapshot } from '../lib/bodies'
import type { Draft } from '../lib/draft'

// Owns the Rust SimulationManager and the JS-side view of its data.
class Sim {
  // Latest body data pulled from Rust. Only refreshed while time runs (and on
  // first load / after adding a body); the renderer just draws whatever is here.
  bodies: BodyData[] = $state.raw([])
  running = $state(false)
  time = $state(0) // seconds

  #lastNow: number | null = null
  #mng: SimulationManager | null = null

  // Created on first use rather than at import: the WASM module is only
  // ready after `await init()` in main.ts.
  #manager(): SimulationManager {
    return (this.#mng ??= new SimulationManager())
  }

  sync() {
    this.bodies = snapshot(this.#manager())
  }

  // Runs once per rendered frame, only while time is running.
  // Advances the clock and pulls fresh positions; stepping the physics goes here later.
  tick(now: number) {
    if (this.#lastNow !== null) this.time += (now - this.#lastNow) / 1000
    this.#lastNow = now
    this.sync()
  }

  toggle() {
    this.running = !this.running
    this.#lastNow = null // don't count the paused time when resuming
  }

  // Throws (with Rust's error message) if the body is rejected.
  addBody(d: Draft) {
    this.#manager().add_body(d.x, d.y, d.vx, d.vy, d.mass, d.radius)
    this.sync()
  }
}

export const sim = new Sim()
