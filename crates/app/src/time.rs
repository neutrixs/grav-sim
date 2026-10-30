use crate::units::{format_sig, to_precision};

const MIN: f64 = 60.0;
const HOUR: f64 = 3600.0;
const DAY: f64 = 86400.0;
const YEAR: f64 = 365.25 * DAY; // Julian year

// Simulation clock: "12.345 s", "01:02:03", "15 d 04:12:33",
// "3 yr 15 d 04:12:33", and from 1000 years on just "12.35 kyr", "2.94 Gyr".
pub fn format_clock(t: f64) -> String {
    if t < MIN {
        return format!("{t:.3} s");
    }
    if t >= 1000.0 * YEAR {
        return format_years(t);
    }
    let total = t.floor();
    let years = (total / YEAR).floor();
    let rest = total - years * YEAR; // whole seconds: a year is a whole number of quarter-days
    let days = (rest / DAY).floor();
    let hms = format!(
        "{:02}:{:02}:{:02}",
        (rest / HOUR).floor() as u64 % 24,
        (rest / MIN).floor() as u64 % 60,
        rest as u64 % 60
    );
    if years > 0.0 {
        format!("{years} yr {days} d {hms}")
    } else if days > 0.0 {
        format!("{days} d {hms}")
    } else {
        hms
    }
}

// A duration in its most readable unit: "1 ms", "86.4 s", "2.5 h", "10 Myr"
pub fn format_duration(s: f64) -> String {
    let n = |v: f64| format!("{}", to_precision(v, 3));
    if s < 1.0 {
        format!("{} ms", n(s * 1000.0))
    } else if s < MIN {
        format!("{} s", n(s))
    } else if s < HOUR {
        format!("{} min", n(s / MIN))
    } else if s < DAY {
        format!("{} h", n(s / HOUR))
    } else if s < YEAR {
        format!("{} d", n(s / DAY))
    } else {
        format_years(s)
    }
}

// Years and up: "12.5 yr", "12.35 kyr", "4.567 Myr", "2.94 Gyr", past a
// million Gyr in e-notation ("1.23e7 Gyr").
fn format_years(s: f64) -> String {
    let y = s / YEAR;
    if y >= 1e9 {
        format!("{} Gyr", format_sig(y / 1e9))
    } else if y >= 1e6 {
        format!("{} Myr", format_sig(y / 1e6))
    } else if y >= 1e3 {
        format!("{} kyr", format_sig(y / 1e3))
    } else {
        format!("{} yr", format_sig(y))
    }
}
