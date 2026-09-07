use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn list_backups() -> Result<Vec<BackupEntry>, RimeError> {
    run_blocking(list_backups_sync).await
}

#[tauri::command]
pub(crate) async fn create_backup(note: Option<String>) -> Result<BackupEntry, RimeError> {
    run_blocking(move || create_backup_with_note_sync(note)).await
}

#[tauri::command]
pub(crate) async fn preview_backup(backup_name: String) -> Result<ConfigPreview, RimeError> {
    run_blocking(move || preview_backup_sync(backup_name)).await
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
