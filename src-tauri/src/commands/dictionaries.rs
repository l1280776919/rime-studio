use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn list_dictionaries() -> Result<Vec<DictInfo>, RimeError> {
    run_blocking(list_dictionaries_sync).await
}

#[tauri::command]
pub(crate) async fn get_dict_health(dict_name: String) -> Result<DictHealth, RimeError> {
    run_blocking(move || get_dict_health_sync(dict_name)).await
}

#[tauri::command]
pub(crate) async fn clean_dictionary_duplicates(
    dict_name: String,
) -> Result<DictionaryCleanResult, RimeError> {
    run_blocking(move || clean_dictionary_duplicates_sync(dict_name)).await
}

#[tauri::command]
pub(crate) async fn delete_dictionary(dict_name: String) -> Result<(), RimeError> {
    run_blocking(move || delete_dictionary_sync(dict_name)).await
}

#[tauri::command]
pub(crate) async fn get_dictionary_config() -> Result<DictionaryConfig, RimeError> {
    run_blocking(read_dictionary_config_sync).await
}

#[tauri::command]
pub(crate) async fn add_dictionary_to_current_schema(
    reference: String,
) -> Result<DictionaryConfig, RimeError> {
    run_blocking(move || add_dictionary_to_current_schema_sync(reference)).await
}

#[tauri::command]
pub(crate) async fn remove_dictionary_from_current_schema(
    reference: String,
) -> Result<DictionaryConfig, RimeError> {
    run_blocking(move || remove_dictionary_from_current_schema_sync(reference)).await
}

#[tauri::command]
pub(crate) async fn save_dictionary_imports(
    imports: Vec<String>,
) -> Result<DictionaryConfig, RimeError> {
    run_blocking(move || save_dictionary_imports_sync(imports)).await
}

#[tauri::command]
pub(crate) async fn list_online_dictionaries() -> Result<Vec<OnlineDictionary>, RimeError> {
    run_blocking(list_online_dictionaries_sync).await
}

#[tauri::command]
pub(crate) async fn list_online_dictionary_categories(
) -> Result<Vec<OnlineDictionaryCategory>, RimeError> {
    run_blocking(list_online_dictionary_categories_sync).await
}

#[tauri::command]
pub(crate) async fn list_online_dictionaries_by_category(
    category_id: String,
) -> Result<Vec<OnlineDictionary>, RimeError> {
    run_blocking(move || list_online_dictionaries_by_category_sync(category_id)).await
}

#[tauri::command]
pub(crate) async fn preview_online_dictionary_import(
    id: String,
) -> Result<DictionaryImportPreview, RimeError> {
    run_blocking(move || preview_online_dictionary_import_sync(id)).await
}

#[tauri::command]
pub(crate) async fn import_online_dictionary(
    id: String,
) -> Result<DictionaryImportResult, RimeError> {
    run_blocking(move || import_online_dictionary_sync(id)).await
}

#[tauri::command]
pub(crate) async fn preview_dictionary_url_import(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportPreview, RimeError> {
    run_blocking(move || preview_dictionary_url_import_sync(url, source_name)).await
}

#[tauri::command]
pub(crate) async fn import_dictionary_url(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportResult, RimeError> {
    run_blocking(move || import_dictionary_url_sync(url, source_name)).await
}

#[tauri::command]
pub(crate) async fn install_lmdg_dicts(
    window: tauri::Window,
) -> Result<LmdgInstallResult, RimeError> {
    run_blocking(move || {
        install_lmdg_dicts_sync_with_progress(|downloaded, total| {
            emit_download_progress(&window, "dicts", "下载万象词库包", downloaded, total);
        })
    })
    .await
}

#[tauri::command]
pub(crate) async fn install_lmdg_grammar(
    window: tauri::Window,
) -> Result<LmdgGrammarInstallResult, RimeError> {
    run_blocking(move || {
        install_lmdg_grammar_sync_with_progress(|downloaded, total| {
            emit_download_progress(&window, "grammar", "下载万象语言模型", downloaded, total);
        })
    })
    .await
}

#[tauri::command]
pub(crate) async fn uninstall_lmdg_grammar() -> Result<LmdgGrammarUninstallResult, RimeError> {
    run_blocking(uninstall_lmdg_grammar_sync).await
}

#[tauri::command]
pub(crate) async fn preview_dictionary_import(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportPreview, RimeError> {
    run_blocking(move || preview_dictionary_import_sync(source_name, data)).await
}

#[tauri::command]
pub(crate) async fn import_dictionary(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportResult, RimeError> {
    run_blocking(move || import_dictionary_sync(source_name, data)).await
}

#[tauri::command]
pub(crate) async fn export_dictionary(
    dict_name: String,
) -> Result<DictionaryExportResult, RimeError> {
    run_blocking(move || export_dictionary_sync(dict_name)).await
}
