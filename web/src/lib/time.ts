import { formatSig } from './units'

const MIN = 60
const HOUR = 3600
const DAY = 86400
const YEAR = 365.25 * DAY // Julian year

const pad = (n: number) => String(n).padStart(2, '0')

// Simulation clock: "12.345 s", "01:02:03", "15 d 04:12:33",
// "3 yr 15 d 04:12:33", and from 1000 years on just "12.35 kyr", "2.94 Gyr".
export function formatClock(t: number): string {
  if (t < MIN) return `${t.toFixed(3)} s`
  if (t >= 1000 * YEAR) return formatYears(t)
  const total = Math.floor(t)
  const years = Math.floor(total / YEAR)
  const rest = total - years * YEAR // whole seconds: a year is a whole number of quarter-days
  const days = Math.floor(rest / DAY)
  const hms = `${pad(Math.floor(rest / HOUR) % 24)}:${pad(Math.floor(rest / MIN) % 60)}:${pad(rest % 60)}`
  if (years > 0) return `${years} yr ${days} d ${hms}`
  return days > 0 ? `${days} d ${hms}` : hms
}

// A duration in its most readable unit: "1 ms", "86.4 s", "2.5 h", "10 Myr"
export function formatDuration(s: number): string {
  const n = (v: number) => Number(v.toPrecision(3)).toString()
  if (s < 1) return `${n(s * 1000)} ms`
  if (s < MIN) return `${n(s)} s`
  if (s < HOUR) return `${n(s / MIN)} min`
  if (s < DAY) return `${n(s / HOUR)} h`
  if (s < YEAR) return `${n(s / DAY)} d`
  return formatYears(s)
}

// Years and up: "12.5 yr", "12.35 kyr", "4.567 Myr", "2.94 Gyr", past a
// million Gyr in e-notation ("1.23e7 Gyr").
function formatYears(s: number): string {
  const y = s / YEAR
  if (y >= 1e9) return `${formatSig(y / 1e9)} Gyr`
  if (y >= 1e6) return `${formatSig(y / 1e6)} Myr`
  if (y >= 1e3) return `${formatSig(y / 1e3)} kyr`
  return `${formatSig(y)} yr`
}
