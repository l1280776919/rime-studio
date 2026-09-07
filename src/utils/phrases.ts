import type { PhraseEntry } from "../types";

export function phraseKey(entry: PhraseEntry) {
  return `${entry.text.trim()}\t${entry.code.trim()}`.toLowerCase();
}

export function countDuplicatePhrases(phrases: PhraseEntry[]) {
  const seen = new Set<string>();
  let duplicates = 0;
  for (const phrase of phrases) {
    const key = phraseKey(phrase);
    if (!key.trim()) continue;
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
    if (!key.trim()) continue;
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
