use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn list_backups() -> Result<Vec<BackupEntry>, RimeError> {
    run_blocking(list_backups_sync).await
}

#[tauri::command]
pub(crate) async fn create_backup() -> Result<BackupEntry, RimeError> {
    run_blocking(create_backup_sync).await
}

#[tauri::command]
pub(crate) async fn open_backup_dir(backup_name: String) -> Result<(), RimeError> {
    run_blocking(move || open_backup_dir_sync(backup_name)).await
}

#[tauri::command]
pub(crate) async fn restore_backup(backup_name: String) -> Result<RestoreResult, RimeError> {
    run_blocking(move || restore_backup_sync(backup_name)).await
}

#[tauri::command]
pub(crate) async fn delete_backup(backup_name: String) -> Result<(), RimeError> {
    run_blocking(move || delete_backup_sync(backup_name)).await
}
