type SearchableCommand = {
  id: string;
  title: string;
  subtitle?: string;
  keywords?: string;
  category: string;
};
export function searchCommands<T extends SearchableCommand>(
  commands: T[],
  query: string,
  recent: string[],
  category = "all",
) {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const filtered = commands.filter(
    (item) =>
      (category === "all" || item.category === category) &&
      words.every((word) =>
        `${item.title} ${item.subtitle ?? ""} ${item.keywords ?? ""} ${item.id} ${item.category}`
          .toLowerCase()
          .includes(word),
      ),
  );
  if (words.length)
    return filtered.sort(
      (a, b) =>
        Number(!words.every((word) => a.title.toLowerCase().includes(word))) -
        Number(!words.every((word) => b.title.toLowerCase().includes(word))),
    );
  return filtered.sort((a, b) => {
    const rank = (id: string) => (recent.includes(id) ? recent.indexOf(id) : recent.length);
    return rank(a.id) - rank(b.id);
  });
}
export function parseRecentCommands(raw: string | null, allowed: string[]) {
  try {
    const value: unknown = JSON.parse(raw ?? "[]");
    if (!Array.isArray(value)) return [];
    return [
      ...new Set(
        value.filter((id): id is string => typeof id === "string" && allowed.includes(id)),
      ),
    ].slice(0, 5);
  } catch {
    return [];
  }
}
export function rememberCommand(recent: string[], id: string) {
  return [id, ...recent.filter((item) => item !== id)].slice(0, 5);
}
