use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn check_app_update() -> Result<AppUpdateInfo, RimeError> {
    run_blocking(check_app_update_sync).await
}

#[tauri::command]
pub(crate) async fn download_app_update() -> Result<RimeDownloadResult, RimeError> {
    run_blocking(download_app_update_sync).await
}

#[tauri::command]
pub(crate) async fn download_rime_installer() -> Result<RimeDownloadResult, RimeError> {
    run_blocking(download_rime_installer_sync).await
}

#[tauri::command]
pub(crate) async fn download_git_installer() -> Result<RimeDownloadResult, RimeError> {
    run_blocking(download_git_installer_sync).await
}

#[tauri::command]
pub(crate) async fn launch_rime_installer(path: String) -> Result<(), RimeError> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[tauri::command]
pub(crate) async fn launch_git_installer(path: String) -> Result<(), RimeError> {
    run_blocking(move || launch_installer_sync(path)).await
}
