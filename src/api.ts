import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { formatInvokeError } from "./utils/error";

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (err) {
    throw new Error(formatInvokeError(err), { cause: err });
  }
}
import type {
  AppearanceConfig,
  AppUpdateInfo,
  BackupEntry,
  CommunitySchema,
  ConfigHealthReport,
  ConfigPreview,
  DictionaryCleanResult,
  DictionaryConfig,
  DictionaryExportResult,
  DictionaryImportPreview,
  DictionaryImportResult,
  DictHealth,
  DictInfo,
  DeployResult,
  FileStatus,
  InstallResult,
  LmdgGrammarInstallResult,
  LmdgGrammarUninstallResult,
  LmdgInstallResult,
  LuaPluginInfo,
  OnlineDictionary,
  OnlineDictionaryCategory,
  PhraseEntry,
  QuickSettingsConfig,
  RestoreResult,
  RimeDownloadResult,
  RimeEnvironment,
  RimeIceSettings,
  RimeSyncConfig,
  SchemaInfo,
  UserdbEntriesResult,
} from "./types";

export const api = {
  scanEnvironment: () => invoke<RimeEnvironment>("scan_rime_environment"),
  scanDictionaryHealth: () => invoke<DictHealth | null>("scan_dictionary_health"),
  deploy: () => invoke<DeployResult>("deploy_rime"),
  cancelDeploy: () => invoke("cancel_deploy"),
  installRimeIce: (recipe?: string) => invoke<InstallResult>("install_rime_ice", { recipe }),

  getAppearance: () => invoke<AppearanceConfig>("get_appearance_config"),
  saveAppearance: (config: AppearanceConfig) =>
    invoke<AppearanceConfig>("save_appearance_config", { config }),
  previewAppearance: (config: AppearanceConfig) =>
    invoke<ConfigPreview>("preview_appearance_config", { config }),
  listSystemFonts: () => invoke<string[]>("list_system_fonts"),

  getQuickSettings: () => invoke<QuickSettingsConfig>("get_quick_settings"),
  saveQuickSettings: (config: QuickSettingsConfig) =>
    invoke<QuickSettingsConfig>("save_quick_settings", { config }),
  previewQuickSettings: (config: QuickSettingsConfig) =>
    invoke<ConfigPreview>("preview_quick_settings", { config }),
  inspectConfigHealth: () => invoke<ConfigHealthReport>("inspect_config_health"),
  repairConfigHealth: () => invoke<ConfigHealthReport>("repair_config_health"),
  repairConfigHealthItem: (name: string) =>
    invoke<ConfigHealthReport>("repair_config_health_item", { name }),
  getRimeIceSettings: () => invoke<RimeIceSettings>("get_rime_ice_settings"),
  saveRimeIceSettings: (settings: RimeIceSettings) =>
    invoke<RimeIceSettings>("save_rime_ice_settings", { settings }),
  previewRimeIceSettings: (settings: RimeIceSettings) =>
    invoke<ConfigPreview>("preview_rime_ice_settings", { settings }),

  listBackups: () => invoke<BackupEntry[]>("list_backups"),
  createBackup: (note?: string) => invoke<BackupEntry>("create_backup", { note }),
  previewBackup: (backupName: string) => invoke<ConfigPreview>("preview_backup", { backupName }),
  openBackupDir: (backupName: string) => invoke("open_backup_dir", { backupName }),
  restoreBackup: (backupName: string) => invoke<RestoreResult>("restore_backup", { backupName }),
  deleteBackup: (backupName: string) => invoke("delete_backup", { backupName }),

  openRimeUserDir: () => invoke("open_rime_user_dir"),
  openSyncDir: () => invoke("open_sync_dir"),
  openPlumDir: () => invoke("open_plum_dir"),
  openConfigFile: (name: string) => invoke("open_config_file", { name }),
  openAppLogDir: () => invoke<string>("open_app_log_dir"),
  restartWeaselServer: () => invoke<string>("restart_weasel_server"),
  getSyncConfig: () => invoke<RimeSyncConfig>("get_sync_config"),
  saveSyncConfig: (installationId?: string, syncDir?: string) =>
    invoke<void>("save_sync_config", { installationId, syncDir }),
  syncRime: () => invoke<string>("sync_rime"),
  listUserdbEntries: (filename: string, limit: number, offset: number, query?: string) =>
    invoke<UserdbEntriesResult>("list_userdb_entries", { filename, limit, offset, query }),

  getCustomPhrases: () => invoke<PhraseEntry[]>("get_custom_phrases"),
  saveCustomPhrases: (phrases: PhraseEntry[]) => invoke("save_custom_phrases", { phrases }),

  listDictionaries: () => invoke<DictInfo[]>("list_dictionaries"),
  getDictionaryConfig: () => invoke<DictionaryConfig>("get_dictionary_config"),
  getDictHealth: (dictName: string) => invoke<DictHealth>("get_dict_health", { dictName }),
  previewDictionaryImport: (sourceName: string, data: number[]) =>
    invoke<DictionaryImportPreview>("preview_dictionary_import", { sourceName, data }),
  listOnlineDictionaries: () => invoke<OnlineDictionary[]>("list_online_dictionaries"),
  listOnlineDictionaryCategories: () =>
    invoke<OnlineDictionaryCategory[]>("list_online_dictionary_categories"),
  listOnlineDictionariesByCategory: (categoryId: string) =>
    invoke<OnlineDictionary[]>("list_online_dictionaries_by_category", { categoryId }),
  installLmdgDicts: () => invoke<LmdgInstallResult>("install_lmdg_dicts"),
  installLmdgGrammar: () => invoke<LmdgGrammarInstallResult>("install_lmdg_grammar"),
  uninstallLmdgGrammar: () => invoke<LmdgGrammarUninstallResult>("uninstall_lmdg_grammar"),
  previewDictionaryUrlImport: (url: string, sourceName?: string) =>
    invoke<DictionaryImportPreview>("preview_dictionary_url_import", { url, sourceName }),
  importOnlineDictionary: (id: string) =>
    invoke<DictionaryImportResult>("import_online_dictionary", { id }),
  importDictionaryUrl: (url: string, sourceName?: string) =>
    invoke<DictionaryImportResult>("import_dictionary_url", { url, sourceName }),
  importDictionary: (sourceName: string, data: number[]) =>
    invoke<DictionaryImportResult>("import_dictionary", { sourceName, data }),
  exportDictionary: (dictName: string) =>
    invoke<DictionaryExportResult>("export_dictionary", { dictName }),
  addDictionaryToCurrentSchema: (reference: string) =>
    invoke<DictionaryConfig>("add_dictionary_to_current_schema", { reference }),
  removeDictionaryFromCurrentSchema: (reference: string) =>
    invoke<DictionaryConfig>("remove_dictionary_from_current_schema", { reference }),
  saveDictionaryImports: (imports: string[]) =>
    invoke<DictionaryConfig>("save_dictionary_imports", { imports }),
  cleanDictionaryDuplicates: (dictName: string) =>
    invoke<DictionaryCleanResult>("clean_dictionary_duplicates", { dictName }),
  deleteDictionary: (dictName: string) => invoke("delete_dictionary", { dictName }),

  checkAppUpdate: () => invoke<AppUpdateInfo>("check_app_update"),
  downloadAppUpdate: () => invoke<RimeDownloadResult>("download_app_update"),
  downloadRimeInstaller: () => invoke<RimeDownloadResult>("download_rime_installer"),
  downloadGitInstaller: () => invoke<RimeDownloadResult>("download_git_installer"),
  launchRimeInstaller: (path: string) => invoke("launch_rime_installer", { path }),
  launchGitInstaller: (path: string) => invoke("launch_git_installer", { path }),

  listSchemas: () => invoke<SchemaInfo[]>("list_schemas"),
  listCommunitySchemas: () => invoke<CommunitySchema[]>("list_community_schemas"),
  copySchema: (schemaId: string) => invoke<string>("copy_schema", { schemaId }),
  setActiveSchema: (schemaId: string) =>
    invoke<QuickSettingsConfig>("set_active_schema", { schemaId }),
  saveActiveSchemaList: (schemaIds: string[]) =>
    invoke<QuickSettingsConfig>("save_active_schema_list", { schemaIds }),
  openSchemaFile: (path: string) => invoke("open_schema_file", { path }),
  openSchemaDir: (path: string) => invoke("open_schema_dir", { path }),

  listLuaPlugins: () => invoke<LuaPluginInfo[]>("list_lua_plugins"),
  toggleLuaPlugin: (pluginId: string, enabled: boolean) =>
    invoke<LuaPluginInfo[]>("toggle_lua_plugin", { pluginId, enabled }),
  getLuaScriptContent: (pluginId: string) => invoke<string>("get_lua_script_content", { pluginId }),
  saveLuaScriptContent: (pluginId: string, content: string) =>
    invoke("save_lua_script_content", { pluginId, content }),

  listYamlConfigFiles: () => invoke<FileStatus[]>("list_yaml_config_files"),
  readConfigFileContent: (filename: string) =>
    invoke<string>("read_config_file_content", { filename }),
  writeConfigFileContent: (filename: string, content: string) =>
    invoke<boolean>("write_config_file_content", { filename, content }),
};
