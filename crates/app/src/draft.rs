use crate::sim::Sim;
use crate::units::to_precision;

// A body being set up in the UI. Lives only on the UI side until it's added.
#[derive(Clone)]
pub struct Draft {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    // Polar form of (vx, vy), kept in sync by the setters below. The angle is
    // stored rather than only derived, so it survives the speed going through 0.
    pub speed: f64, // m/s
    pub angle: f64, // degrees, 0 = +x, counter-clockwise
    pub mass: f64,
    pub radius: f64,
}

// Trim float noise (e.g. 4.999999999999999 → 5) so the fields stay readable.
fn clean(v: f64) -> f64 {
    to_precision(v, 12)
}

impl Draft {
    // vx/vy changed → recompute speed and angle.
    pub fn set_velocity_xy(&mut self, vx: f64, vy: f64) {
        self.vx = vx;
        self.vy = vy;
        if !vx.is_finite() || !vy.is_finite() {
            return;
        }
        self.speed = clean(vx.hypot(vy));
        if self.speed > 0.0 {
            self.angle = clean(vy.atan2(vx).to_degrees().rem_euclid(360.0));
        }
    }

    // Speed changed → keep the angle.
    pub fn set_speed(&mut self, speed: f64) {
        if !speed.is_finite() || speed < 0.0 {
            return;
        }
        self.speed = speed;
        self.polar_to_xy();
    }

    // Angle changed → keep the speed.
    pub fn set_angle(&mut self, angle: f64) {
        if !angle.is_finite() {
            return;
        }
        self.angle = angle;
        self.polar_to_xy();
    }

    fn polar_to_xy(&mut self) {
        let rad = self.angle.to_radians();
        // cos(90°) is 6e-17 in floats, not 0: snap tiny components to exact 0
        let tidy = |v: f64| if v.abs() < self.speed * 1e-12 { 0.0 } else { clean(v) };
        self.vx = tidy(self.speed * rad.cos());
        self.vy = tidy(self.speed * rad.sin());
    }
}

// Placing: ghost follows the cursor, click drops it
// Aiming:  velocity follows the cursor, click sets it
// Editing: drag the body / arrow tip, or type in the fields
#[derive(Clone, Copy, PartialEq)]
pub enum DraftMode {
    Placing,
    Aiming,
    Editing,
}

// The body being set up on the canvas / in the panel. Only sent to the core on confirm.
pub struct DraftState {
    pub draft: Option<Draft>,
    pub mode: DraftMode,
    pub error: String,
    // New drafts start with the mass/radius of the last added body.
    last_mass: f64,
    last_radius: f64,
}

impl Default for DraftState {
    fn default() -> Self {
        DraftState { draft: None, mode: DraftMode::Placing, error: String::new(), last_mass: 1.0, last_radius: 1.0 }
    }
}

impl DraftState {
    pub fn start(&mut self, x: f64, y: f64) {
        self.draft = Some(Draft {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            speed: 0.0,
            angle: 0.0,
            mass: self.last_mass,
            radius: self.last_radius,
        });
        self.mode = DraftMode::Placing;
        self.error.clear();
    }

    pub fn cancel(&mut self) {
        self.draft = None;
        self.error.clear();
    }

    pub fn confirm(&mut self, sim: &mut Sim) {
        let Some(d) = &self.draft else { return };
        if ![d.x, d.y, d.vx, d.vy, d.mass, d.radius].iter().all(|v| v.is_finite()) {
            self.error = "All fields must be numbers".into();
            return;
        }
        match sim.add_body(d) {
            Ok(()) => {
                self.last_mass = d.mass;
                self.last_radius = d.radius;
                self.cancel();
            }
            Err(e) => self.error = e,
        }
    }
}
