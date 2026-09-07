export type FileStatus = {
  name: string;
  path: string;
  exists: boolean;
  size?: number;
  modified?: number;
};

export type DictHealth = {
  entries: number;
  duplicate_exact_lines: number;
  long_low_weight_entries: number;
  truncated?: boolean;
};

export type RimeEnvironment = {
  user_dir: string;
  build_dir: string;
  deployer_path?: string;
  plum_dir: string;
  git_available: boolean;
  bash_available: boolean;
  git_path?: string;
  bash_path?: string;
  active_schema?: string;
  page_size?: number;
  theme_name?: string;
  font_point?: number;
  label_font_point?: number;
  custom_files: FileStatus[];
  sogou_health?: DictHealth;
  user_dicts: UserDictInfo[];
  sync_dir: FileStatus;
};

export type UserDictInfo = {
  name: string;
  path: string;
  size_bytes: number;
  modified?: number;
};

export type DeployProgress = {
  stage: string;
  log: string;
  elapsed_ms: number;
};

export type DeployResult = {
  success: boolean;
  message: string;
  log?: string;
  hints?: string[];
  duration_ms?: number;
};

export type InstallResult = {
  success: boolean;
  recipe: string;
  backup_dir?: string;
  log: string;
};

export type BackupEntry = {
  name: string;
  path: string;
  kind: "manual" | "before-save" | "before-restore" | "before-install" | string;
  modified?: number;
  files: number;
  scope?: string;
  note?: string;
};

export type RestoreResult = {
  restored_files: number;
  safety_backup_dir: string;
};

export type ColorScheme = {
  name: string;
  label: string;
  back_color: string;
  border_color: string;
  text_color: string;
  candidate_text_color: string;
  comment_text_color: string;
  hilited_text_color: string;
  hilited_back_color: string;
  hilited_candidate_text_color: string;
  hilited_candidate_back_color: string;
};

export type AppearanceConfig = {
  theme_name: string;
  font_point: number;
  label_font_point: number;
  font_face?: string;
  label_font_face?: string;
  page_size: number;
  switch_key: string;
  horizontal: boolean;
  inline_preedit: boolean;
  candidate_format: string;
  corner_radius: number;
  border_height: number;
  border_width: number;
  line_spacing: number;
  spacing: number;
  back_color: string;
  border_color: string;
  text_color: string;
  candidate_text_color: string;
  comment_text_color: string;
  hilited_text_color: string;
  hilited_back_color: string;
  hilited_candidate_text_color: string;
  hilited_candidate_back_color: string;
  custom_schemes?: ColorScheme[];
};

export type QuickSettingsConfig = {
  schema_id: string;
  page_size: number;
  switch_key: string;
  paging_keys: string;
  navigation_keys: string;
  horizontal: boolean;
  inline_preedit: boolean;
};

export type ConfigHealthCheck = {
  name: string;
  status: "ok" | "warning" | "error";
  detail: string;
};

export type ConfigHealthReport = {
  summary: string;
  checks: ConfigHealthCheck[];
};

export type ConfigPreview = {
  files: {
    name: string;
    path: string;
    changed: boolean;
    diff_lines: string[];
  }[];
};

export type RimeIceSettings = {
  emoji: boolean;
  traditionalization: boolean;
  ascii_punct: boolean;
  full_shape: boolean;
  search_single_char: boolean;
  fuzzy_pinyin: boolean;
  fuzzy_pairs?: string[];
  traditional_preset: string;
};

export type PhraseEntry = {
  text: string;
  code: string;
  weight: number;
  tag?: string;
};

export type SchemaInfo = {
  id: string;
  name: string;
  description: string;
  path: string;
  is_system: boolean;
  is_active: boolean;
  is_enabled: boolean;
};

export type DictInfo = {
  name: string;
  path: string;
  entry_count: number;
  size_bytes: number;
  modified?: number;
};

export type DictionaryImportResult = {
  name: string;
  reference: string;
  path: string;
  imported_entries: number;
  skipped_entries: number;
};

export type DictionaryImportPreview = {
  name: string;
  reference: string;
  path: string;
  imported_entries: number;
  skipped_entries: number;
  sample_entries: { text: string; code: string; weight: number }[];
  will_overwrite: boolean;
};

export type DictionaryExportResult = {
  name: string;
  contents: string;
};

export type DictionaryCleanResult = {
  name: string;
  path: string;
  removed_duplicate_lines: number;
  entries_after: number;
  backup_dir?: string;
};

export type DictionaryReference = {
  reference: string;
  path?: string;
  exists: boolean;
  entry_count?: number;
  size_bytes?: number;
};

export type DictionaryConfig = {
  schema_id?: string;
  schema_name?: string;
  main_dictionary?: string;
  main_dictionary_path?: string;
  enabled: DictionaryReference[];
  available: DictInfo[];
  missing: DictionaryReference[];
};

export type OnlineDictionary = {
  id: string;
  title: string;
  category: string;
  description: string;
  source: string;
  source_name: string;
  detail_url: string;
};

export type OnlineDictionaryCategory = {
  id: string;
  title: string;
  description: string;
};

export type LmdgInstallResult = {
  installed_count: number;
  target_dir: string;
  source_url: string;
  message: string;
};

export type LmdgGrammarInstallResult = {
  model_name: string;
  model_path: string;
  patch_path: string;
  source_url: string;
  message: string;
};

export type LmdgGrammarUninstallResult = {
  model_name: string;
  model_path: string;
  patch_path: string;
  removed_model: boolean;
  message: string;
};

export type LmdgDownloadProgress = {
  kind: "grammar" | "dicts";
  stage: string;
  downloaded_bytes: number;
  total_bytes?: number;
  percent?: number;
};

export type AppUpdateInfo = {
  current_version: string;
  latest_version?: string;
  release_name?: string;
  release_notes?: string;
  published_at?: string;
  release_url: string;
  asset_name?: string;
  asset_url?: string;
  update_available: boolean;
};

export type LuaPluginInfo = {
  id: string;
  name: string;
  description: string;
  file_name: string;
  trigger_preview: string;
  installed: boolean;
  enabled: boolean;
  author: string;
};

export type CommunitySchema = {
  id: string;
  name: string;
  description: string;
  recipe: string;
  author: string;
  tags: string[];
  installed: boolean;
};

export type RimeDownloadResult = {
  success: boolean;
  installer_path?: string;
  message: string;
};

export type UserdbSnapshotInfo = {
  name: string;
  file_size: number;
  modified?: number;
  entry_count: number;
};

export type RimeSyncConfig = {
  installation_id: string;
  sync_dir?: string;
  resolved_sync_dir: string;
  last_sync_time?: number;
  snapshot_files: UserdbSnapshotInfo[];
};

export type UserdbEntry = {
  word: string;
  code: string;
  count: number;
};

export type UserdbEntriesResult = {
  entries: UserdbEntry[];
  total: number;
};
