export function formatBytes(value?: number) {
  if (value === undefined || value === null) return "—";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

export function formatTime(value?: number) {
  if (!value) return "\u2014";
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

export function backupLabel(backup: { name: string }): string {
  const raw = backup.name
    .replace("backup-rime-studio-before-save-", "")
    .replace("backup-rime-studio-before-restore-", "")
    .replace("backup-rime-studio-before-install-", "")
    .replace("backup-rime-studio-manual-", "")
    .replace("backup-rime-studio-", "");
  return raw.replace(/[-_]/g, " ");
}

export function backupKindLabel(kind: string): string {
  if (kind === "before-save") return "\u4fdd\u5b58\u524d";
  if (kind === "before-restore") return "\u6062\u590d\u524d";
  if (kind === "before-install") return "\u5b89\u88c5\u524d";
  return "\u624b\u52a8";
}

export function backupKindType(kind: string): "info" | "warning" | "success" {
  if (kind === "before-save") return "info";
  if (kind === "before-restore") return "warning";
  if (kind === "before-install") return "warning";
  return "success";
}
