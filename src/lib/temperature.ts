/**
 * Formats a temperature value from Celsius to the user's preferred unit.
 * Centralizes the conversion logic used across MemoryTab, SystemInfoTab, etc.
 */
export function formatTemperature(
  celsius: number,
  unit: string | undefined
): string {
  if (unit === "f") {
    return `${((celsius * 9) / 5 + 32).toFixed(0)}°F`;
  }
  return `${celsius.toFixed(0)}°C`;
}

/**
 * Returns a Tailwind color class for a temperature value (Celsius).
 */
export function tempColorClass(celsius: number): string {
  if (celsius > 80) return "bg-red-500/10 text-red-400 border-red-500/20";
  if (celsius > 60) return "bg-amber-500/10 text-amber-400 border-amber-500/20";
  return "bg-emerald-500/10 text-emerald-400 border-emerald-500/20";
}
