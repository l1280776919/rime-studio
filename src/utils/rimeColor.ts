/** Weasel COLORREF is 0xBBGGRR (and optionally 0xAABBGGRR). CSS wants #RRGGBB. */

export function rimeToCssColor(value?: string, fallback = "#000000"): string {
  if (!value) return fallback;
  const cleaned = value.trim();
  if (cleaned.startsWith("#")) {
    if (/^#[0-9A-Fa-f]{6}$/.test(cleaned)) return cleaned;
    if (/^#[0-9A-Fa-f]{3}$/.test(cleaned)) {
      const [, r, g, b] = cleaned;
      return `#${r}${r}${g}${g}${b}${b}`;
    }
    return fallback;
  }

  const hex = cleaned.replace(/^0x/i, "");
  if (!/^[0-9A-Fa-f]{6}$/.test(hex) && !/^[0-9A-Fa-f]{8}$/.test(hex)) {
    return fallback;
  }

  const bgr = hex.slice(-6).toUpperCase();
  const rgb = `${bgr.slice(4, 6)}${bgr.slice(2, 4)}${bgr.slice(0, 2)}`;
  if (hex.length === 8) {
    const alpha = hex.slice(0, 2).toUpperCase();
    if (alpha !== "FF") {
      const a = Number.parseInt(alpha, 16) / 255;
      const r = Number.parseInt(rgb.slice(0, 2), 16);
      const g = Number.parseInt(rgb.slice(2, 4), 16);
      const b = Number.parseInt(rgb.slice(4, 6), 16);
      return `rgba(${r}, ${g}, ${b}, ${Number.isFinite(a) ? a.toFixed(3) : 1})`;
    }
  }
  return `#${rgb}`;
}

export function cssToRimeColor(value: string): string {
  const n = value.replace(/^#/, "").padStart(6, "0").slice(-6);
  if (!/^[0-9A-Fa-f]{6}$/.test(n)) return "0x000000";
  const bgr = `${n.slice(4, 6)}${n.slice(2, 4)}${n.slice(0, 2)}`.toUpperCase();
  return `0x${bgr}`;
}

export function cssFontFamily(face?: string): string {
  const trimmed = face?.trim();
  if (!trimmed) return "var(--font-sans)";
  return trimmed
    .split(",")
    .map((part) => {
      const name = part.trim().replace(/^["']|["']$/g, "");
      if (!name) return "";
      return /[\s\d]/.test(name) ? `"${name}"` : name;
    })
    .filter(Boolean)
    .join(", ");
}

export function formatCandidateLabel(
  format: string | undefined,
  index: number,
  text: string,
  comment = "",
): { label: string; text: string; comment: string } {
  const template = format?.trim() || "%c. %@";
  const rendered = template
    .replaceAll("%c", String(index + 1))
    .replaceAll("%@", text)
    .replaceAll("%s", comment);
  return { label: `${index + 1}.`, text: rendered, comment };
}
