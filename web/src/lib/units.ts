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
const MASS: Unit[] = [
  { label: 'g', factor: 1e-3 },
  { label: 'kg', factor: 1 },
  { label: 't', factor: 1e3 },
]
const ANGLE: Unit[] = [
  { label: '°', factor: 1 },
  { label: 'rad', factor: 180 / Math.PI },
]

// Fields in the same group share one unit choice (x and y, vx/vy/|v|, …).
export const UNIT_GROUPS = {
  position: LENGTH,
  radius: LENGTH,
  velocity: SPEED,
  angle: ANGLE,
  mass: MASS,
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
