// Display units. Values are always stored in the base unit (SI, except angles,
// which the draft keeps in degrees); units only change what's shown and typed.
#[derive(Clone, Copy, PartialEq)]
pub struct Unit {
    pub label: &'static str,
    pub factor: f64, // stored value = shown value × factor
}

const fn u(label: &'static str, factor: f64) -> Unit {
    Unit { label, factor }
}

const AU: f64 = 1.495978707e11; // m, astronomical unit (exact by definition)
const LY: f64 = 9.4607304725808e15; // m, light-year (Julian year × c)
const YEAR: f64 = 31557600.0; // s, Julian year (365.25 d)

const LENGTH: &[Unit] = &[u("mm", 1e-3), u("cm", 1e-2), u("m", 1.0), u("km", 1e3), u("AU", AU)];

// Reference bodies, used as units ("2 Earth" = two Earth masses) and as
// presets in the mass and radius fields. Radii are mean radii (Sun: IAU nominal).
pub const BODY_MASSES: &[Unit] =
    &[u("Moon", 7.342e22), u("Earth", 5.972e24), u("Jupiter", 1.898e27), u("Sun", 1.989e30)];
pub const BODY_RADII: &[Unit] =
    &[u("Moon", 1.7374e6), u("Earth", 6.371e6), u("Jupiter", 6.9911e7), u("Sun", 6.957e8)];

const RADIUS: &[Unit] = &[
    u("mm", 1e-3),
    u("cm", 1e-2),
    u("m", 1.0),
    u("km", 1e3),
    u("AU", AU),
    u("Moon", 1.7374e6),
    u("Earth", 6.371e6),
    u("Jupiter", 6.9911e7),
    u("Sun", 6.957e8),
];
const SPEED: &[Unit] = &[u("m/s", 1.0), u("km/h", 1.0 / 3.6), u("km/s", 1e3)];
const MASS: &[Unit] = &[
    u("g", 1e-3),
    u("kg", 1.0),
    u("t", 1e3),
    u("Moon", 7.342e22),
    u("Earth", 5.972e24),
    u("Jupiter", 1.898e27),
    u("Sun", 1.989e30),
];
const ANGLE: &[Unit] = &[u("°", 1.0), u("rad", 180.0 / std::f64::consts::PI)];
const DURATION: &[Unit] =
    &[u("ms", 1e-3), u("s", 1.0), u("min", 60.0), u("h", 3600.0), u("d", 86400.0)];
// Playback speed: simulated time per real second.
const TIMESCALE: &[Unit] = &[
    u("s/s", 1.0),
    u("min/s", 60.0),
    u("h/s", 3600.0),
    u("d/s", 86400.0),
    u("yr/s", YEAR),
    u("kyr/s", YEAR * 1e3),
    u("Myr/s", YEAR * 1e6),
    u("Gyr/s", YEAR * 1e9),
];

// Fields in the same group share one unit choice (x and y, vx/vy/|v|, …).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitGroup {
    Position,
    Radius,
    Velocity,
    Angle,
    Mass,
    Dt,
    Speed,
}

impl UnitGroup {
    pub fn units(self) -> &'static [Unit] {
        match self {
            UnitGroup::Position => LENGTH,
            UnitGroup::Radius => RADIUS,
            UnitGroup::Velocity => SPEED,
            UnitGroup::Angle => ANGLE,
            UnitGroup::Mass => MASS,
            UnitGroup::Dt => DURATION,
            UnitGroup::Speed => TIMESCALE,
        }
    }
}

// Display unit chosen per group, shared by every field and readout of that group.
pub struct UnitPrefs([&'static str; 7]);

impl Default for UnitPrefs {
    fn default() -> Self {
        UnitPrefs(["m", "m", "m/s", "°", "kg", "ms", "s/s"])
    }
}

impl UnitPrefs {
    pub fn label(&self, g: UnitGroup) -> &'static str {
        self.0[g as usize]
    }
    pub fn set(&mut self, g: UnitGroup, label: &'static str) {
        self.0[g as usize] = label;
    }
    pub fn unit_of(&self, g: UnitGroup) -> Unit {
        let options = g.units();
        *options.iter().find(|u| u.label == self.label(g)).unwrap_or(&options[0])
    }
}

// Like JS toPrecision: round to p significant digits.
pub fn to_precision(v: f64, p: usize) -> f64 {
    if !v.is_finite() || v == 0.0 {
        return v;
    }
    format!("{:.*e}", p - 1, v).parse().unwrap_or(v)
}

// Non-finite values (an empty field) pass through untouched.
pub fn to_display(stored: f64, u: Unit) -> f64 {
    if stored.is_finite() { to_precision(stored / u.factor, 12) } else { stored }
}

pub fn from_display(shown: f64, u: Unit) -> f64 {
    if shown.is_finite() { to_precision(shown * u.factor, 15) } else { shown }
}

// "1234567.5" → "1 234 567.5"; `sep` between groups of three integer digits.
pub fn group_digits(s: &str, sep: char) -> String {
    let (sign, rest) = s.strip_prefix('-').map_or(("", s), |r| ("-", r));
    let (int, frac) = rest.split_once('.').map_or((rest, None), |(i, f)| (i, Some(f)));
    let mut out = String::from(sign);
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(sep);
        }
        out.push(c);
    }
    if let Some(f) = frac {
        out.push('.');
        out.push_str(f);
    }
    out
}

// Fixed decimals with trailing zeros (and a bare ".") removed.
fn trimmed(v: f64, decimals: usize) -> String {
    let s = format!("{v:.decimals$}");
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
}

// 4 significant figures, thousands grouped with spaces: "384 400", "0.1069".
// Values past a million switch to e-notation ("1.23e13") so they stay short.
pub fn format_sig(v: f64) -> String {
    if v.abs() >= 1e6 {
        return format!("{v:.2e}");
    }
    if v == 0.0 {
        return "0".into();
    }
    let decimals = 3 - v.abs().log10().floor() as i32;
    let s = if decimals >= 0 {
        trimmed(v, decimals as usize)
    } else {
        let q = 10f64.powi(-decimals);
        format!("{:.0}", (v / q).round() * q)
    };
    group_digits(&s, ' ')
}

// Read-only distance with its unit picked to fit the size:
// 384400000 → "384 400 km", 1.6e10 → "0.1069 AU", 1.5e20 → "15.86 kly".
pub fn format_distance(m: f64) -> String {
    let a = m.abs();
    let (v, label) = if a >= 1e9 * LY {
        (m / LY / 1e9, "Gly")
    } else if a >= 1e6 * LY {
        (m / LY / 1e6, "Mly")
    } else if a >= 1e3 * LY {
        (m / LY / 1e3, "kly")
    } else if a >= LY {
        (m / LY, "ly")
    } else if a >= 1e9 {
        (m / AU, "AU") // past 1 million km, so km values stay short
    } else if a >= 1e3 {
        (m / 1e3, "km")
    } else if a >= 1.0 || a == 0.0 {
        (m, "m")
    } else if a >= 1e-2 {
        (m * 1e2, "cm")
    } else {
        (m * 1e3, "mm")
    };
    format!("{} {label}", format_sig(v))
}

// Grid spacing label: "1 m", "10 km", "1 000 000 km", "1 cm", ...
pub fn format_length(m: f64) -> String {
    let nf = |v: f64| group_digits(&trimmed(v, 3), ' ');
    if m >= 1e3 {
        format!("{} km", nf(m / 1e3))
    } else if m >= 1.0 {
        format!("{} m", nf(m))
    } else if m >= 1e-2 {
        format!("{} cm", nf(m * 1e2))
    } else {
        format!("{} mm", nf(m * 1e3))
    }
}

// Text in an editable number field: plain digits up to 1e12, e-notation beyond.
pub fn num_text(v: f64) -> String {
    if v.abs() >= 1e12 { format!("{v:e}") } else { format!("{v}") }
}
