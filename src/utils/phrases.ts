import type { PhraseEntry } from "../types";

export function phraseKey(entry: PhraseEntry) {
  return JSON.stringify([entry.text, entry.code]);
}

export function countDuplicatePhrases(phrases: PhraseEntry[]) {
  const seen = new Set<string>();
  let duplicates = 0;
  for (const phrase of phrases) {
    const key = phraseKey(phrase);
    if (!phrase.text.trim()) continue;
    if (seen.has(key)) {
      duplicates++;
    } else {
      seen.add(key);
    }
  }
  return duplicates;
}

export function dedupePhrases(phrases: PhraseEntry[]) {
  const byKey = new Map<string, PhraseEntry>();
  for (const phrase of phrases) {
    const key = phraseKey(phrase);
    if (!phrase.text.trim()) continue;
    const current = byKey.get(key);
    if (!current || phrase.weight > current.weight) {
      byKey.set(key, { ...phrase });
    }
  }
  return Array.from(byKey.values());
}

export function paginateItems<T>(items: T[], page: number, pageSize: number) {
  const size = Math.max(1, pageSize);
  const totalPages = Math.max(1, Math.ceil(items.length / size) || 1);
  const current = Math.min(Math.max(1, page), totalPages);
  const start = (current - 1) * size;
  return {
    page: current,
    totalPages,
    items: items.slice(start, start + size),
  };
}

export function isValidPhrase(entry: PhraseEntry) {
  const text = entry.text.trim().replace(/^\uFEFF/, "");
  return (
    !!text &&
    !text.startsWith("#") &&
    text !== "---" &&
    text !== "..." &&
    !/[\t\r\n\0]/.test(entry.text + entry.code) &&
    Number.isInteger(entry.weight) &&
    entry.weight >= -2147483648 &&
    entry.weight <= 2147483647
  );
}

export function parsePhraseImport(contents: string): PhraseEntry[] {
  return contents.split(/\r?\n/).flatMap((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#") || trimmed === "---" || trimmed === "...") return [];
    const parts = line.includes("\t")
      ? line.split("\t")
      : line.includes(",")
        ? line.split(",")
        : trimmed.split(/\s+/);
    if (parts.length > 3) return [];
    const entry = {
      text: parts[0],
      code: parts[1] ?? "",
      weight: parts[2]?.trim() ? Number(parts[2]) : 1,
    };
    return isValidPhrase(entry) ? [entry] : [];
  });
}
