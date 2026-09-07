export function formatInvokeError(err: unknown): string {
  if (err instanceof Error) {
    const parsed = tryParseErrorPayload(err.message);
    if (parsed) return parsed;
    return err.message;
  }
  if (typeof err === "string") {
    return tryParseErrorPayload(err) ?? err;
  }
  if (err && typeof err === "object") {
    const payload = err as { message?: unknown; code?: unknown };
    if (typeof payload.message === "string" && payload.message.trim()) {
      return payload.message;
    }
  }
  return String(err);
}

function tryParseErrorPayload(value: string): string | undefined {
  const trimmed = value.trim();
  if (!trimmed.startsWith("{")) return undefined;
  try {
    const parsed = JSON.parse(trimmed) as { message?: unknown };
    if (typeof parsed.message === "string" && parsed.message.trim()) {
      return parsed.message;
    }
  } catch {
    return undefined;
  }
  return undefined;
}
