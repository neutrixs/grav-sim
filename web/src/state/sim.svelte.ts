import { Sim as RustSim } from '../../pkg'
import { type BodyData, snapshot } from '../lib/bodies'
import type { Draft } from '../lib/draft'

// fixed:   dt is set directly
// dynamic: dt = dtFactor × speed, so faster playback takes bigger steps
//          (each step always covers dtFactor seconds of *real* time)
// auto:    like dynamic, but the factor follows the measured cost of a step,
//          so the physics uses about AUTO_LOAD of real time
export type StepMode = 'fixed' | 'dynamic' | 'auto'

const valid = (v: number) => Number.isFinite(v) && v > 0

// A frame never asks for more than this much real time: after a long stall
// the sim runs slower instead of trying to catch up (and stalling again).
const MAX_FRAME_S = 0.05
// Real time per frame the physics may spend; above it, steps are dropped.
const STEP_BUDGET_MS = 10
// How often the slowdown readout is recomputed.
const SLOWDOWN_WINDOW_S = 0.5
// Auto mode: fraction of real time spent stepping. Kept below the frame
// budget (STEP_BUDGET_MS of a ~16 ms frame) so steps aren't dropped.
const AUTO_LOAD = 0.4

type DtSettings = Pick<Sim, 'speed' | 'stepMode' | 'fixedDt' | 'dtFactor' | 'autoFactor'>
function dtOf(s: DtSettings): number {
  if (s.stepMode === 'fixed') return s.fixedDt
  return (s.stepMode === 'auto' ? s.autoFactor : s.dtFactor) * s.speed
}

// Owns the Rust Sim and the JS-side view of its data.
class Sim {
  // Latest body data pulled from Rust. Only refreshed while time runs (and on
  // first load / after adding a body); the renderer just draws whatever is here.
  bodies: BodyData[] = $state.raw([])
  running = $state(false)
  time = $state(0) // simulated seconds, as reported by Rust after each step
  slowdown = $state(1) // how many times slower than `speed` the sim actually ran

  speed = $state(1) // simulated seconds per real second
  stepMode: StepMode = $state('dynamic')
  fixedDt = $state(0.001) // s
  dtFactor = $state(0.001) // s of real time per step
  autoFactor = $state(0.001) // same, but picked by auto mode

  // The step size currently handed to Rust (s).
  get dt(): number {
    return dtOf(this)
  }

  #lastNow: number | null = null
  #mng: RustSim | null = null

  // Simulated time requested but smaller than one step, carried to the next
  // frame. Kept in seconds (not steps) so it stays right when dt changes.
  #owed = 0
  // Measured real cost of one step (ms), smoothed; caps steps per frame.
  #msPerStep = 0
  // Slowdown window: simulated time asked for vs. dropped.
  #winReal = 0
  #winWanted = 0
  #winDropped = 0

  // Created on first use rather than at import: the WASM module is only
  // ready after `await init()` in main.ts.
  #manager(): RustSim {
    return (this.#mng ??= new RustSim(this.dt))
  }

  // Apply a settings change only if Rust accepts the dt it results in
  // (setting dt throws on NaN/∞, ≤ 0, or a step too small for fixed-point).
  // A rejected change is dropped, and the field snaps back to the current value.
  #commit(change: Partial<Pick<Sim, 'speed' | 'stepMode' | 'fixedDt' | 'dtFactor'>>) {
    const next = { speed: this.speed, stepMode: this.stepMode, fixedDt: this.fixedDt, dtFactor: this.dtFactor, ...change }
    const dt = dtOf({ ...next, autoFactor: this.autoFactor })
    try {
      this.#manager().dt = dt
    } catch {
      return
    }
    Object.assign(this, next)
  }

  setSpeed(v: number) {
    // speed isn't part of dt in fixed mode, so Rust can't catch a bad one there
    if (!valid(v)) return
    this.#commit({ speed: v })
  }
  setStepMode(m: StepMode) {
    this.#commit({ stepMode: m })
  }
  setFixedDt(v: number) {
    this.#commit({ fixedDt: v })
  }
  setDtFactor(v: number) {
    this.#commit({ dtFactor: v })
  }

  sync() {
    this.bodies = snapshot(this.#manager())
  }

  // Runs once per rendered frame, only while time is running.
  // Works out how many steps this frame's real time is worth, runs them in
  // Rust, then takes the clock and positions from Rust. JS never advances the
  // clock on its own, so it always matches the bodies on screen.
  tick(now: number) {
    if (this.#lastNow === null) {
      this.#lastNow = now // first frame after play: just start measuring
      return
    }
    const real = (now - this.#lastNow) / 1000
    this.#lastNow = now

    const wanted = real * this.speed
    let dropped = Math.max(0, real - MAX_FRAME_S) * this.speed

    // Whole steps that fit in the requested time; the remainder carries over.
    this.#owed += Math.min(real, MAX_FRAME_S) * this.speed
    let n = Math.floor(this.#owed / this.dt)

    // Don't ask for more steps than fit in the frame budget (or in an i32).
    const maxSteps = Math.min(
      this.#msPerStep > 0 ? Math.max(1, Math.floor(STEP_BUDGET_MS / this.#msPerStep)) : 1000,
      2 ** 31 - 1,
    )
    if (n > maxSteps) {
      dropped += (n - maxSteps) * this.dt
      n = maxSteps
      this.#owed = 0 // the rest is dropped, not owed: falling behind is never paid back
    } else {
      this.#owed -= n * this.dt
    }

    if (n > 0) {
      const t0 = performance.now()
      const rs = this.#manager()
      rs.step(n)
      this.time = rs.time
      const ms = (performance.now() - t0) / n
      this.#msPerStep = this.#msPerStep > 0 ? this.#msPerStep * 0.8 + ms * 0.2 : ms
      this.sync()
    }

    if (this.stepMode === 'auto' && this.#msPerStep > 0) this.#autoAdjust()

    this.#measureSlowdown(real, wanted, dropped)
  }

  // Each step should cost (dt / speed) × AUTO_LOAD of real time, so
  // dt / speed = cost of a step / AUTO_LOAD. #msPerStep is already smoothed.
  #autoAdjust() {
    const factor = this.#msPerStep / 1000 / AUTO_LOAD
    try {
      this.#manager().dt = factor * this.speed
    } catch {
      return // too small for fixed-point: keep the current one
    }
    this.autoFactor = factor
  }

  // slowdown = wanted / actually simulated, over a short window so a single
  // slow frame doesn't make the readout jump.
  #measureSlowdown(real: number, wanted: number, dropped: number) {
    this.#winReal += real
    this.#winWanted += wanted
    this.#winDropped += dropped
    if (this.#winReal < SLOWDOWN_WINDOW_S) return

    const done = this.#winWanted - this.#winDropped
    this.slowdown = done > 0 ? this.#winWanted / done : Infinity
    this.#winReal = this.#winWanted = this.#winDropped = 0
  }

  toggle() {
    this.running = !this.running
    // don't count the paused time when resuming
    this.#lastNow = null
    this.#winReal = this.#winWanted = this.#winDropped = 0
    this.slowdown = 1
  }

  // Throws (with Rust's error message) if the body is rejected.
  addBody(d: Draft) {
    this.#manager().add_body(d.x, d.y, d.vx, d.vy, d.mass, d.radius)
    this.sync()
  }
}

export const sim = new Sim()
