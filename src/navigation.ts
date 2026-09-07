export const PAGE_KEYS = [
  "overview",
  "quick",
  "schemas",
  "editor",
  "appearance",
  "phrases",
  "dictionaries",
  "backups",
  "about",
] as const;

export type PageKey = (typeof PAGE_KEYS)[number];

export function isPageKey(value: string): value is PageKey {
  return (PAGE_KEYS as readonly string[]).includes(value);
}
