pub(crate) mod types;
pub(crate) use types::*;

pub(crate) mod backend;
pub(crate) use backend::*;

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("后台任务失败: {err}"))?
}

#[tauri::command]
async fn scan_rime_environment() -> Result<RimeEnvironment, String> {
    run_blocking(scan_rime_environment_sync).await
}

#[tauri::command]
async fn deploy_rime() -> Result<DeployResult, String> {
    run_blocking(deploy_rime_sync).await
}

#[tauri::command]
async fn install_rime_ice(recipe: Option<String>) -> Result<InstallResult, String> {
    run_blocking(move || install_rime_ice_sync(recipe)).await
}

#[tauri::command]
async fn check_app_update() -> Result<AppUpdateInfo, String> {
    run_blocking(check_app_update_sync).await
}

#[tauri::command]
async fn download_app_update() -> Result<RimeDownloadResult, String> {
    run_blocking(download_app_update_sync).await
}

#[tauri::command]
async fn get_appearance_config() -> Result<AppearanceConfig, String> {
    run_blocking(get_appearance_config_sync).await
}

#[tauri::command]
async fn get_quick_settings() -> Result<QuickSettingsConfig, String> {
    run_blocking(get_quick_settings_sync).await
}

#[tauri::command]
async fn save_quick_settings(config: QuickSettingsConfig) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || save_quick_settings_sync(config)).await
}

#[tauri::command]
async fn preview_quick_settings(config: QuickSettingsConfig) -> Result<ConfigPreview, String> {
    run_blocking(move || preview_quick_settings_sync(config)).await
}

#[tauri::command]
async fn inspect_config_health() -> Result<ConfigHealthReport, String> {
    run_blocking(inspect_config_health_sync).await
}

#[tauri::command]
async fn repair_config_health() -> Result<ConfigHealthReport, String> {
    run_blocking(repair_config_health_sync).await
}

#[tauri::command]
async fn repair_config_health_item(name: String) -> Result<ConfigHealthReport, String> {
    run_blocking(move || repair_config_health_item_sync(name)).await
}

#[tauri::command]
async fn get_rime_ice_settings() -> Result<RimeIceSettings, String> {
    run_blocking(get_rime_ice_settings_sync).await
}

#[tauri::command]
async fn save_rime_ice_settings(settings: RimeIceSettings) -> Result<RimeIceSettings, String> {
    run_blocking(move || save_rime_ice_settings_sync(settings)).await
}

#[tauri::command]
async fn save_appearance_config(config: AppearanceConfig) -> Result<AppearanceConfig, String> {
    run_blocking(move || save_appearance_config_sync(config)).await
}

#[tauri::command]
async fn list_backups() -> Result<Vec<BackupEntry>, String> {
    run_blocking(list_backups_sync).await
}

#[tauri::command]
async fn create_backup() -> Result<BackupEntry, String> {
    run_blocking(create_backup_sync).await
}

#[tauri::command]
async fn open_rime_user_dir() -> Result<(), String> {
    run_blocking(open_rime_user_dir_sync).await
}

#[tauri::command]
async fn open_config_file(name: String) -> Result<(), String> {
    run_blocking(move || open_config_file_sync(name)).await
}

#[tauri::command]
async fn open_plum_dir() -> Result<(), String> {
    run_blocking(open_plum_dir_sync).await
}

#[tauri::command]
async fn open_backup_dir(backup_name: String) -> Result<(), String> {
    run_blocking(move || open_backup_dir_sync(backup_name)).await
}

#[tauri::command]
async fn restore_backup(backup_name: String) -> Result<RestoreResult, String> {
    run_blocking(move || restore_backup_sync(backup_name)).await
}

#[tauri::command]
async fn delete_backup(backup_name: String) -> Result<(), String> {
    run_blocking(move || delete_backup_sync(backup_name)).await
}

#[tauri::command]
async fn delete_dictionary(dict_name: String) -> Result<(), String> {
    run_blocking(move || delete_dictionary_sync(dict_name)).await
}

#[tauri::command]
async fn get_custom_phrases() -> Result<Vec<PhraseEntry>, String> {
    run_blocking(get_custom_phrases_sync).await
}

#[tauri::command]
async fn save_custom_phrases(phrases: Vec<PhraseEntry>) -> Result<(), String> {
    run_blocking(move || save_custom_phrases_sync(phrases)).await
}

#[tauri::command]
async fn list_dictionaries() -> Result<Vec<DictInfo>, String> {
    run_blocking(scan_dictionaries_sync_wrapper).await
}

#[tauri::command]
async fn get_dict_health(dict_name: String) -> Result<DictHealth, String> {
    run_blocking(move || get_dict_health_sync_wrapper(dict_name)).await
}

#[tauri::command]
async fn clean_dictionary_duplicates(dict_name: String) -> Result<DictionaryCleanResult, String> {
    run_blocking(move || clean_dictionary_duplicates_sync(dict_name)).await
}

#[tauri::command]
async fn get_dictionary_config() -> Result<DictionaryConfig, String> {
    run_blocking(read_dictionary_config_sync).await
}

#[tauri::command]
async fn add_dictionary_to_current_schema(reference: String) -> Result<DictionaryConfig, String> {
    run_blocking(move || add_dictionary_to_current_schema_sync(reference)).await
}

#[tauri::command]
async fn remove_dictionary_from_current_schema(
    reference: String,
) -> Result<DictionaryConfig, String> {
    run_blocking(move || remove_dictionary_from_current_schema_sync(reference)).await
}

#[tauri::command]
async fn save_dictionary_imports(imports: Vec<String>) -> Result<DictionaryConfig, String> {
    run_blocking(move || save_dictionary_imports_sync(imports)).await
}

#[tauri::command]
async fn list_online_dictionaries() -> Result<Vec<OnlineDictionary>, String> {
    run_blocking(list_online_dictionaries_sync).await
}

#[tauri::command]
async fn list_online_dictionary_categories() -> Result<Vec<OnlineDictionaryCategory>, String> {
    run_blocking(list_online_dictionary_categories_sync).await
}

#[tauri::command]
async fn list_online_dictionaries_by_category(
    category_id: String,
) -> Result<Vec<OnlineDictionary>, String> {
    run_blocking(move || list_online_dictionaries_by_category_sync(category_id)).await
}

#[tauri::command]
async fn preview_online_dictionary_import(id: String) -> Result<DictionaryImportPreview, String> {
    run_blocking(move || preview_online_dictionary_import_sync(id)).await
}

#[tauri::command]
async fn import_online_dictionary(id: String) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_online_dictionary_sync(id)).await
}

#[tauri::command]
async fn preview_dictionary_url_import(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportPreview, String> {
    run_blocking(move || preview_dictionary_url_import_sync(url, source_name)).await
}

#[tauri::command]
async fn import_dictionary_url(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_dictionary_url_sync(url, source_name)).await
}

#[tauri::command]
async fn install_lmdg_dicts(window: tauri::Window) -> Result<LmdgInstallResult, String> {
    run_blocking(move || {
        install_lmdg_dicts_sync_with_progress(|downloaded, total| {
            emit_download_progress(&window, "dicts", "下载万象词库包", downloaded, total);
        })
    })
    .await
}

#[tauri::command]
async fn install_lmdg_grammar(window: tauri::Window) -> Result<LmdgGrammarInstallResult, String> {
    run_blocking(move || {
        install_lmdg_grammar_sync_with_progress(|downloaded, total| {
            emit_download_progress(&window, "grammar", "下载万象语言模型", downloaded, total);
        })
    })
    .await
}

#[tauri::command]
async fn uninstall_lmdg_grammar() -> Result<LmdgGrammarUninstallResult, String> {
    run_blocking(uninstall_lmdg_grammar_sync).await
}

#[tauri::command]
async fn preview_dictionary_import(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportPreview, String> {
    run_blocking(move || preview_dictionary_import_sync(source_name, data)).await
}

#[tauri::command]
async fn import_dictionary(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_dictionary_sync(source_name, data)).await
}

#[tauri::command]
async fn export_dictionary(dict_name: String) -> Result<DictionaryExportResult, String> {
    run_blocking(move || export_dictionary_sync(dict_name)).await
}

#[tauri::command]
async fn download_rime_installer() -> Result<RimeDownloadResult, String> {
    run_blocking(download_rime_installer_sync).await
}

#[tauri::command]
async fn download_git_installer() -> Result<RimeDownloadResult, String> {
    run_blocking(download_git_installer_sync).await
}

#[tauri::command]
async fn launch_rime_installer(path: String) -> Result<(), String> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[tauri::command]
async fn launch_git_installer(path: String) -> Result<(), String> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[tauri::command]
async fn list_schemas() -> Result<Vec<SchemaInfo>, String> {
    run_blocking(list_schemas_sync).await
}

#[tauri::command]
async fn copy_schema(schema_id: String) -> Result<String, String> {
    run_blocking(move || copy_schema_sync(schema_id)).await
}

#[tauri::command]
async fn set_active_schema(schema_id: String) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || set_active_schema_sync(schema_id)).await
}

#[tauri::command]
async fn save_active_schema_list(schema_ids: Vec<String>) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || save_active_schema_list_sync(schema_ids)).await
}

#[tauri::command]
async fn open_schema_file(path: String) -> Result<(), String> {
    run_blocking(move || open_schema_file_sync(path)).await
}

#[tauri::command]
async fn open_schema_dir(path: String) -> Result<(), String> {
    run_blocking(move || open_schema_dir_sync(path)).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_rime_environment,
            deploy_rime,
            install_rime_ice,
            check_app_update,
            download_app_update,
            get_appearance_config,
            get_quick_settings,
            save_quick_settings,
            preview_quick_settings,
            inspect_config_health,
            repair_config_health,
            repair_config_health_item,
            get_rime_ice_settings,
            save_rime_ice_settings,
            save_appearance_config,
            list_backups,
            create_backup,
            open_rime_user_dir,
            open_config_file,
            open_plum_dir,
            open_backup_dir,
            restore_backup,
            delete_backup,
            delete_dictionary,
            get_custom_phrases,
            save_custom_phrases,
            list_dictionaries,
            get_dict_health,
            clean_dictionary_duplicates,
            get_dictionary_config,
            add_dictionary_to_current_schema,
            remove_dictionary_from_current_schema,
            save_dictionary_imports,
            list_online_dictionaries,
            list_online_dictionary_categories,
            list_online_dictionaries_by_category,
            preview_online_dictionary_import,
            import_online_dictionary,
            preview_dictionary_url_import,
            import_dictionary_url,
            install_lmdg_dicts,
            install_lmdg_grammar,
            uninstall_lmdg_grammar,
            preview_dictionary_import,
            import_dictionary,
            export_dictionary,
            download_rime_installer,
            download_git_installer,
            launch_rime_installer,
            launch_git_installer,
            list_schemas,
            copy_schema,
            set_active_schema,
            save_active_schema_list,
            open_schema_file,
            open_schema_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
