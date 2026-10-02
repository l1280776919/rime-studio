export type FileRevision = { content: string | null; initialContent?: string };
export type ConflictChoice = "overwrite" | "reload" | "keep";
export type GuardedResult =
  | { kind: "saved"; revision: FileRevision }
  | { kind: "reload"; revision: FileRevision }
  | { kind: "keep" };

/** Preserve structured IPC errors, including the cause attached by api.invoke. */
function isConflict(error: unknown): boolean {
  if (error instanceof Error) return isConflict(error.cause);
  return (
    !!error && typeof error === "object" && "code" in error && error.code === "config_conflict"
  );
}

/** Every overwrite is conditional on the latest reviewed revision, never forced. */
export async function saveWithConflict(
  expected: FileRevision,
  read: () => Promise<FileRevision>,
  write: (revision: FileRevision) => Promise<FileRevision>,
  choose: (previous: FileRevision, current: FileRevision) => Promise<ConflictChoice>,
): Promise<GuardedResult> {
  let revision = expected;
  for (;;) {
    try {
      return { kind: "saved", revision: await write(revision) };
    } catch (error) {
      if (!isConflict(error)) throw error;
      const current = await read();
      const choice = await choose(revision, current);
      if (choice === "keep") return { kind: "keep" };
      if (choice === "reload") return { kind: "reload", revision: current };
      revision = current;
    }
  }
}
