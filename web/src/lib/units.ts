// Display units. Values are always stored in the base unit (SI, except angles,
// which the draft keeps in degrees); units only change what's shown and typed.
export interface Unit {
  label: string
  factor: number // stored value = shown value × factor
}

const LENGTH: Unit[] = [
  { label: 'mm', factor: 1e-3 },
  { label: 'cm', factor: 1e-2 },
  { label: 'm', factor: 1 },
  { label: 'km', factor: 1e3 },
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
]

// Fields in the same group share one unit choice (x and y, vx/vy/|v|, …).
export const UNIT_GROUPS = {
  position: LENGTH,
  radius: LENGTH,
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
