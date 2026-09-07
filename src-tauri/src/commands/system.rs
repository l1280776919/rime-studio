use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn scan_rime_environment() -> Result<RimeEnvironment, RimeError> {
    run_blocking(scan_rime_environment_sync).await
}

#[tauri::command]
pub(crate) async fn deploy_rime(app: tauri::AppHandle) -> Result<DeployResult, RimeError> {
    run_blocking(move || deploy_rime_sync(Some(app))).await
}

#[tauri::command]
pub(crate) async fn cancel_deploy() -> Result<(), RimeError> {
    request_cancel_deploy();
    Ok(())
}

#[tauri::command]
pub(crate) async fn open_sync_dir() -> Result<(), RimeError> {
    run_blocking(open_sync_dir_sync).await
}

#[tauri::command]
pub(crate) async fn install_rime_ice(recipe: Option<String>) -> Result<InstallResult, RimeError> {
    run_blocking(move || install_rime_ice_sync(recipe)).await
}

#[tauri::command]
pub(crate) async fn open_rime_user_dir() -> Result<(), RimeError> {
    run_blocking(open_rime_user_dir_sync).await
}

#[tauri::command]
pub(crate) async fn open_plum_dir() -> Result<(), RimeError> {
    run_blocking(open_plum_dir_sync).await
}

#[tauri::command]
pub(crate) async fn open_config_file(name: String) -> Result<(), RimeError> {
    run_blocking(move || open_config_file_sync(name)).await
}

#[tauri::command]
pub(crate) async fn scan_dictionary_health() -> Result<Option<DictHealth>, RimeError> {
    run_blocking(scan_dictionary_health_sync).await
}

#[tauri::command]
pub(crate) async fn open_app_log_dir(app: tauri::AppHandle) -> Result<String, RimeError> {
    use tauri::Manager;

    let dir = app
        .path()
        .app_log_dir()
        .map_err(|err| RimeError::FileOperationError(format!("无法定位日志目录: {err}")))?;
    std::fs::create_dir_all(&dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建日志目录失败: {err}")))?;
    open_in_explorer(&dir)?;
    Ok(dir.display().to_string())
}

#[tauri::command]
pub(crate) async fn restart_weasel_server() -> Result<String, RimeError> {
    run_blocking(restart_weasel_server_sync).await
}

#[tauri::command]
pub(crate) async fn get_sync_config() -> Result<RimeSyncConfig, RimeError> {
    run_blocking(get_sync_config_sync).await
}

#[tauri::command]
pub(crate) async fn save_sync_config(
    installation_id: Option<String>,
    sync_dir: Option<String>,
) -> Result<(), RimeError> {
    run_blocking(move || save_sync_config_sync(installation_id, sync_dir)).await
}

#[tauri::command]
pub(crate) async fn sync_rime() -> Result<String, RimeError> {
    run_blocking(sync_rime_sync).await
}

#[tauri::command]
pub(crate) async fn list_userdb_entries(
    filename: String,
    limit: usize,
    offset: usize,
    query: Option<String>,
) -> Result<UserdbEntriesResult, RimeError> {
    run_blocking(move || list_userdb_entries_sync(filename, limit, offset, query)).await
}
