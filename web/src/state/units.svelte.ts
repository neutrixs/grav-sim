import { type Unit, type UnitGroup, UNIT_GROUPS } from '../lib/units'

// Display unit chosen per group, shared by every field and readout of that group.
export const unitPrefs: Record<UnitGroup, string> = $state({
  position: 'm',
  radius: 'm',
  velocity: 'm/s',
  angle: '°',
  mass: 'kg',
})

export function unitOf(group: UnitGroup): Unit {
  const options = UNIT_GROUPS[group]
  return options.find((u) => u.label === unitPrefs[group]) ?? options[0]
}
