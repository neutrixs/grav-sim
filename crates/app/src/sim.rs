use std::time::Instant;

use grav_core::fixed::{FRAC_BITS, Fixed};
use grav_core::sim::{SimError, SimulationManager};
use grav_core::vec::Vector2D;

use crate::draft::Draft;

// fixed:   dt is set directly
// dynamic: dt = dt_factor × speed, so faster playback takes bigger steps
//          (each step always covers dt_factor seconds of *real* time)
// auto:    like dynamic, but the factor follows the measured cost of a step,
//          so the physics uses about AUTO_LOAD of real time
#[derive(Clone, Copy, PartialEq)]
pub enum StepMode {
    Fixed,
    Dynamic,
    Auto,
}

// A frame never asks for more than this much real time: after a long stall
// the sim runs slower instead of trying to catch up (and stalling again).
const MAX_FRAME_S: f64 = 0.05;
// Real time per frame the physics may spend; above it, steps are dropped.
const STEP_BUDGET_MS: f64 = 10.0;
// How often the slowdown readout is recomputed.
const SLOWDOWN_WINDOW_S: f64 = 0.5;
// Auto mode: fraction of real time spent stepping. Kept below the frame
// budget (STEP_BUDGET_MS of a ~16 ms frame) so steps aren't dropped.
const AUTO_LOAD: f64 = 0.4;

// Plain copy of a body, so the UI doesn't hold on to the core's data.
pub struct BodyData {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

// Everything that decides dt, so a change can be checked before it's applied.
#[derive(Clone, Copy)]
pub struct DtSettings {
    pub speed: f64, // simulated seconds per real second
    pub step_mode: StepMode,
    pub fixed_dt: f64,    // s
    pub dt_factor: f64,   // s of real time per step
    pub auto_factor: f64, // same, but picked by auto mode
}

impl DtSettings {
    pub fn dt(&self) -> f64 {
        match self.step_mode {
            StepMode::Fixed => self.fixed_dt,
            StepMode::Dynamic => self.dt_factor * self.speed,
            StepMode::Auto => self.auto_factor * self.speed,
        }
    }
}

// f64 → Fixed, or None if it can't be represented (NaN, ∞, or so large that
// the fixed-point scaling overflows and Fixed::from_f64 would panic).
fn to_fixed(v: f64) -> Option<Fixed> {
    (v * 2f64.powi(FRAC_BITS as i32)).is_finite().then(|| Fixed::from_f64(v))
}

// Owns the core SimulationManager and the UI-side view of its data.
pub struct Sim {
    core: SimulationManager,
    // Latest body data pulled from the core. Only refreshed while time runs
    // (and after adding a body); the renderer just draws whatever is here.
    pub bodies: Vec<BodyData>,
    pub running: bool,
    pub time: f64,     // simulated seconds, as reported by the core after each step
    pub slowdown: f64, // how many times slower than `speed` the sim actually ran
    pub multithread: bool,
    pub cfg: DtSettings,

    last_now: Option<Instant>,
    // Simulated time requested but smaller than one step, carried to the next
    // frame. Kept in seconds (not steps) so it stays right when dt changes.
    owed: f64,
    // Measured real cost of one step (ms), smoothed; caps steps per frame.
    ms_per_step: f64,
    // Slowdown window: simulated time asked for vs. dropped.
    win_real: f64,
    win_wanted: f64,
    win_dropped: f64,
}

impl Sim {
    pub fn new() -> Self {
        let cfg = DtSettings {
            speed: 1.0,
            step_mode: StepMode::Dynamic,
            fixed_dt: 0.001,
            dt_factor: 0.001,
            auto_factor: 0.001,
        };
        let core = SimulationManager::new(Fixed::from_f64(cfg.dt())).expect("default dt is valid");
        Sim {
            core,
            bodies: Vec::new(),
            running: false,
            time: 0.0,
            slowdown: 1.0,
            multithread: false,
            cfg,
            last_now: None,
            owed: 0.0,
            ms_per_step: 0.0,
            win_real: 0.0,
            win_wanted: 0.0,
            win_dropped: 0.0,
        }
    }

    pub fn dt(&self) -> f64 {
        self.cfg.dt()
    }

    // Hand a dt to the core. False if it's rejected (NaN/∞, ≤ 0, or a step
    // too small for fixed-point).
    fn set_core_dt(&mut self, dt: f64) -> bool {
        if dt.is_nan() || dt <= 0.0 {
            return false;
        }
        to_fixed(dt).is_some_and(|f| self.core.set_dt(f).is_ok())
    }

    // Apply a settings change only if the core accepts the dt it results in.
    // A rejected change is dropped, and the field snaps back to the current value.
    fn commit(&mut self, next: DtSettings) {
        if self.set_core_dt(next.dt()) {
            self.cfg = next;
        }
    }

    pub fn set_speed(&mut self, v: f64) {
        // speed isn't part of dt in fixed mode, so the core can't catch a bad one there
        if !(v.is_finite() && v > 0.0) {
            return;
        }
        self.commit(DtSettings { speed: v, ..self.cfg });
    }
    pub fn set_step_mode(&mut self, m: StepMode) {
        self.commit(DtSettings { step_mode: m, ..self.cfg });
    }
    pub fn set_fixed_dt(&mut self, v: f64) {
        self.commit(DtSettings { fixed_dt: v, ..self.cfg });
    }
    pub fn set_dt_factor(&mut self, v: f64) {
        self.commit(DtSettings { dt_factor: v, ..self.cfg });
    }

    pub fn sync(&mut self) {
        self.bodies = self
            .core
            .retrieve_all()
            .iter()
            .map(|b| BodyData {
                id: b.id,
                x: b.pos.x.to_f64(),
                y: b.pos.y.to_f64(),
                radius: b.radius.to_f64(),
            })
            .collect();
    }

    // Runs once per rendered frame, only while time is running.
    // Works out how many steps this frame's real time is worth, runs them in
    // the core, then takes the clock and positions from the core.
    pub fn tick(&mut self, now: Instant) {
        let Some(last) = self.last_now.replace(now) else {
            return; // first frame after play: just start measuring
        };
        let real = (now - last).as_secs_f64();
        let speed = self.cfg.speed;
        let dt = self.dt();

        let wanted = real * speed;
        let mut dropped = (real - MAX_FRAME_S).max(0.0) * speed;

        // Whole steps that fit in the requested time; the remainder carries over.
        self.owed += real.min(MAX_FRAME_S) * speed;
        let mut n = (self.owed / dt).floor();

        // Don't ask for more steps than fit in the frame budget (or in a u32).
        let max_steps = if self.ms_per_step > 0.0 {
            (STEP_BUDGET_MS / self.ms_per_step).floor().max(1.0)
        } else {
            1000.0
        }
        .min(u32::MAX as f64);
        if n > max_steps {
            dropped += (n - max_steps) * dt;
            n = max_steps;
            self.owed = 0.0; // the rest is dropped, not owed: falling behind is never paid back
        } else {
            self.owed -= n * dt;
        }

        if n > 0.0 {
            let n = n as u32;
            let t0 = Instant::now();
            let stepped = if self.multithread { self.core.step_parallel(n) } else { self.core.step(n) };
            self.time = stepped.map(|t| t.to_f64()).unwrap_or(self.time);
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / n as f64;
            self.ms_per_step = if self.ms_per_step > 0.0 { self.ms_per_step * 0.8 + ms * 0.2 } else { ms };
            self.sync();
        }

        if self.cfg.step_mode == StepMode::Auto && self.ms_per_step > 0.0 {
            self.auto_adjust();
        }

        self.measure_slowdown(real, wanted, dropped);
    }

    // Each step should cost (dt / speed) × AUTO_LOAD of real time, so
    // dt / speed = cost of a step / AUTO_LOAD. ms_per_step is already smoothed.
    fn auto_adjust(&mut self) {
        let factor = self.ms_per_step / 1000.0 / AUTO_LOAD;
        if self.set_core_dt(factor * self.cfg.speed) {
            self.cfg.auto_factor = factor;
        } // else too small for fixed-point: keep the current one
    }

    // slowdown = wanted / actually simulated, over a short window so a single
    // slow frame doesn't make the readout jump.
    fn measure_slowdown(&mut self, real: f64, wanted: f64, dropped: f64) {
        self.win_real += real;
        self.win_wanted += wanted;
        self.win_dropped += dropped;
        if self.win_real < SLOWDOWN_WINDOW_S {
            return;
        }
        let done = self.win_wanted - self.win_dropped;
        self.slowdown = if done > 0.0 { self.win_wanted / done } else { f64::INFINITY };
        self.win_real = 0.0;
        self.win_wanted = 0.0;
        self.win_dropped = 0.0;
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
        // don't count the paused time when resuming
        self.last_now = None;
        self.win_real = 0.0;
        self.win_wanted = 0.0;
        self.win_dropped = 0.0;
        self.slowdown = 1.0;
    }

    // Err with the core's (or the validation's) message if the body is rejected.
    pub fn add_body(&mut self, d: &Draft) -> Result<(), String> {
        let err = |e: SimError| e.to_string();
        let (Some(x), Some(y)) = (to_fixed(d.x), to_fixed(d.y)) else {
            return Err(err(SimError::InvalidPosition(d.x, d.y)));
        };
        let (Some(vx), Some(vy)) = (to_fixed(d.vx), to_fixed(d.vy)) else {
            return Err(err(SimError::InvalidVelocity(d.vx, d.vy)));
        };
        let Some(mass) = to_fixed(d.mass).filter(|_| d.mass > 0.0) else {
            return Err(err(SimError::InvalidMass(d.mass)));
        };
        let Some(radius) = to_fixed(d.radius).filter(|_| d.radius > 0.0) else {
            return Err(err(SimError::InvalidRadius(d.radius)));
        };
        self.core
            .add_body(Vector2D { x, y }, Vector2D { x: vx, y: vy }, mass, radius)
            .map_err(err)?;
        self.sync();
        Ok(())
    }
}
