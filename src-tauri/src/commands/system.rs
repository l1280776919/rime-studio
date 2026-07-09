use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn scan_rime_environment() -> Result<RimeEnvironment, RimeError> {
    run_blocking(scan_rime_environment_sync).await
}

#[tauri::command]
pub(crate) async fn deploy_rime() -> Result<DeployResult, RimeError> {
    run_blocking(deploy_rime_sync).await
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
