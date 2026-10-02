use super::run_blocking;
use crate::backend::*;
use crate::types::RimeError;

#[tauri::command]
pub(crate) async fn list_migration_files() -> Result<Vec<MigrationFileInfo>, RimeError> {
    run_blocking(list_migration_files_sync).await
}

#[tauri::command]
pub(crate) async fn export_migration(
    categories: Vec<String>,
) -> Result<MigrationExport, RimeError> {
    run_blocking(move || export_migration_sync(categories)).await
}

#[tauri::command]
pub(crate) async fn preview_migration(
    data: Vec<u8>,
    selected_names: Option<Vec<String>>,
) -> Result<MigrationPreview, RimeError> {
    run_blocking(move || preview_migration_sync(data, selected_names)).await
}

#[tauri::command]
pub(crate) async fn import_migration(token: String) -> Result<MigrationResult, RimeError> {
    run_blocking(move || import_migration_sync(token)).await
}

#[tauri::command]
pub(crate) async fn open_migration_export_dir() -> Result<(), RimeError> {
    run_blocking(open_migration_export_dir_sync).await
}

#[tauri::command]
pub(crate) async fn discard_migration_preview(token: String) -> Result<(), RimeError> {
    run_blocking(move || discard_migration_preview_sync(token)).await
}
