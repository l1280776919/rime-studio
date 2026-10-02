export const MIGRATION_CATEGORIES = [
  { value: "config", label: "配置文件" },
  { value: "schemas", label: "输入方案" },
  { value: "dictionaries", label: "词库" },
  { value: "phrases", label: "自定义短语" },
  { value: "lua", label: "Lua 脚本" },
] as const;
export type MigrationCategory = (typeof MIGRATION_CATEGORIES)[number]["value"];
export type MigrationFile = { name: string; category: MigrationCategory; bytes: number };
export type MigrationExport = { path: string; files: number; bytes: number };
export type MigrationPreviewFile = MigrationFile & {
  status: "new" | "same" | "conflict";
  selected: boolean;
  diff: string[];
};
export type MigrationPreview = {
  token: string;
  app_version: string;
  created_at: string;
  files: MigrationPreviewFile[];
  blockers: string[];
};
export type MigrationResult = { imported_files: number; safety_backup_dir: string };

/** Stable selection keys keep a reviewed plan invalid until the exact selection is restored. */
export function migrationSelectionKey(names: string[]): string {
  return JSON.stringify([...new Set(names)].sort());
}
