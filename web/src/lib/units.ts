// Display units. Values are always stored in the base unit (SI, except angles,
// which the draft keeps in degrees); units only change what's shown and typed.
export interface Unit {
  label: string
  factor: number // stored value = shown value × factor
}

const AU = 1.495978707e11 // m, astronomical unit (exact by definition)
const LY = 9.4607304725808e15 // m, light-year (Julian year × c)

const LENGTH: Unit[] = [
  { label: 'mm', factor: 1e-3 },
  { label: 'cm', factor: 1e-2 },
  { label: 'm', factor: 1 },
  { label: 'km', factor: 1e3 },
  { label: 'AU', factor: AU },
]
// Mean radii of the same reference bodies (Sun: IAU nominal).
export const BODY_RADII: Unit[] = [
  { label: 'Moon', factor: 1.7374e6 },
  { label: 'Earth', factor: 6.371e6 },
  { label: 'Jupiter', factor: 6.9911e7 },
  { label: 'Sun', factor: 6.957e8 },
]
const SPEED: Unit[] = [
  { label: 'm/s', factor: 1 },
  { label: 'km/h', factor: 1 / 3.6 },
  { label: 'km/s', factor: 1e3 },
]
// Reference bodies, used as units ("2 Earth" = two Earth masses) and as
// presets in the mass field.
export const BODY_MASSES: Unit[] = [
  { label: 'Moon', factor: 7.342e22 },
  { label: 'Earth', factor: 5.972e24 },
  { label: 'Jupiter', factor: 1.898e27 },
  { label: 'Sun', factor: 1.989e30 },
]
const MASS: Unit[] = [
  { label: 'g', factor: 1e-3 },
  { label: 'kg', factor: 1 },
  { label: 't', factor: 1e3 },
  ...BODY_MASSES,
]
const ANGLE: Unit[] = [
  { label: '°', factor: 1 },
  { label: 'rad', factor: 180 / Math.PI },
]

const DURATION: Unit[] = [
  { label: 'ms', factor: 1e-3 },
  { label: 's', factor: 1 },
  { label: 'min', factor: 60 },
  { label: 'h', factor: 3600 },
  { label: 'd', factor: 86400 },
]
// Playback speed: simulated time per real second.
const TIMESCALE: Unit[] = [
  { label: 's/s', factor: 1 },
  { label: 'min/s', factor: 60 },
  { label: 'h/s', factor: 3600 },
  { label: 'd/s', factor: 86400 },
  { label: 'yr/s', factor: 31557600 }, // Julian year, 365.25 d
  { label: 'kyr/s', factor: 31557600e3 },
  { label: 'Myr/s', factor: 31557600e6 },
  { label: 'Gyr/s', factor: 31557600e9 },
]

// Fields in the same group share one unit choice (x and y, vx/vy/|v|, …).
export const UNIT_GROUPS = {
  position: LENGTH,
  radius: [...LENGTH, ...BODY_RADII],
  velocity: SPEED,
  angle: ANGLE,
  mass: MASS,
  dt: DURATION,
  speed: TIMESCALE,
} satisfies Record<string, Unit[]>

export type UnitGroup = keyof typeof UNIT_GROUPS

// Non-finite values (an empty field) pass through untouched, so the setters
// downstream can still tell "empty" apart from 0.
export function toDisplay(stored: number, u: Unit): number {
  return Number.isFinite(stored) ? Number((stored / u.factor).toPrecision(12)) : stored
}

export function fromDisplay(shown: number, u: Unit): number {
  return Number.isFinite(shown) ? Number((shown * u.factor).toPrecision(15)) : shown
}

// 4 significant figures, thousands grouped with spaces: "384 400", "0.1069".
// Values past a million switch to e-notation ("1.23e13") so they stay short.
export function formatSig(v: number): string {
  if (Math.abs(v) >= 1e6) return v.toExponential(2).replace('+', '')
  return v.toLocaleString('en-US', { maximumSignificantDigits: 4 }).replace(/,/g, ' ')
}

// Read-only distance with its unit picked to fit the size:
// 384400000 → "384 400 km", 1.6e10 → "0.1069 AU", 1.5e20 → "15.86 kly".
export function formatDistance(m: number): string {
  const a = Math.abs(m)
  const [v, label] =
    a >= 1e9 * LY ? [m / LY / 1e9, 'Gly']
    : a >= 1e6 * LY ? [m / LY / 1e6, 'Mly']
    : a >= 1e3 * LY ? [m / LY / 1e3, 'kly']
    : a >= LY ? [m / LY, 'ly']
    : a >= 1e9 ? [m / AU, 'AU'] // past 1 million km, so km values stay short
    : a >= 1e3 ? [m / 1e3, 'km']
    : a >= 1 || a === 0 ? [m, 'm']
    : a >= 1e-2 ? [m * 1e2, 'cm']
    : [m * 1e3, 'mm']
  return `${formatSig(v)} ${label}`
}
