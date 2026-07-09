use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn list_schemas() -> Result<Vec<SchemaInfo>, RimeError> {
    run_blocking(list_schemas_sync).await
}

#[tauri::command]
pub(crate) async fn copy_schema(schema_id: String) -> Result<String, RimeError> {
    run_blocking(move || copy_schema_sync(schema_id)).await
}

#[tauri::command]
pub(crate) async fn set_active_schema(schema_id: String) -> Result<QuickSettingsConfig, RimeError> {
    run_blocking(move || set_active_schema_sync(schema_id)).await
}

#[tauri::command]
pub(crate) async fn save_active_schema_list(
    schema_ids: Vec<String>,
) -> Result<QuickSettingsConfig, RimeError> {
    run_blocking(move || save_active_schema_list_sync(schema_ids)).await
}

#[tauri::command]
pub(crate) async fn open_schema_file(path: String) -> Result<(), RimeError> {
    run_blocking(move || open_schema_file_sync(path)).await
}

#[tauri::command]
pub(crate) async fn open_schema_dir(path: String) -> Result<(), RimeError> {
    run_blocking(move || open_schema_dir_sync(path)).await
}
