/**
 * Formats an unknown thrown value into a human-readable string.
 * Prevents `[object Object]` appearing in error banners.
 */
export function formatError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return "An unknown error occurred";
}
