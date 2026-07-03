export type FileStatus = {
  name: string;
  path: string;
  exists: boolean;
  size?: number;
};

export type DictHealth = {
  entries: number;
  duplicate_exact_lines: number;
  long_low_weight_entries: number;
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
};

export type DeployResult = {
  success: boolean;
  message: string;
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
  modified?: number;
  files: number;
};

export type RestoreResult = {
  restored_files: number;
  safety_backup_dir: string;
};

export type AppearanceConfig = {
  theme_name: string;
  font_point: number;
  label_font_point: number;
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

export type RimeIceSettings = {
  emoji: boolean;
  traditionalization: boolean;
  ascii_punct: boolean;
  full_shape: boolean;
  search_single_char: boolean;
  traditional_preset: string;
};

export type PhraseEntry = {
  text: string;
  code: string;
  weight: number;
};

export type SchemaInfo = {
  id: string;
  name: string;
  description: string;
  path: string;
  is_system: boolean;
  is_active: boolean;
};

export type DictInfo = {
  name: string;
  path: string;
  entry_count: number;
  size_bytes: number;
  modified?: number;
};
