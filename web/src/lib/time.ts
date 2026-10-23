const MIN = 60
const HOUR = 3600
const DAY = 86400

const pad = (n: number) => String(n).padStart(2, '0')

// Simulation clock: "12.345 s", "01:02:03", "15 d 04:12:33"
export function formatClock(t: number): string {
  if (t < MIN) return `${t.toFixed(3)} s`
  const total = Math.floor(t)
  const hms = `${pad(Math.floor(total / HOUR) % 24)}:${pad(Math.floor(total / MIN) % 60)}:${pad(total % 60)}`
  const days = Math.floor(total / DAY)
  return days > 0 ? `${days} d ${hms}` : hms
}

// A duration in its most readable unit: "1 ms", "86.4 s", "2.5 h"
export function formatDuration(s: number): string {
  const n = (v: number) => Number(v.toPrecision(3)).toString()
  if (s < 1) return `${n(s * 1000)} ms`
  if (s < MIN) return `${n(s)} s`
  if (s < HOUR) return `${n(s / MIN)} min`
  if (s < DAY) return `${n(s / HOUR)} h`
  return `${n(s / DAY)} d`
}
