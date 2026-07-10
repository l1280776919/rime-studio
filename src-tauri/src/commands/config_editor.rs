use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn list_yaml_config_files() -> Result<Vec<FileStatus>, RimeError> {
    run_blocking(list_yaml_config_files_sync).await
}

#[tauri::command]
pub(crate) async fn read_config_file_content(filename: String) -> Result<String, RimeError> {
    run_blocking(move || read_config_file_content_sync(filename)).await
}

#[tauri::command]
pub(crate) async fn write_config_file_content(
    filename: String,
    content: String,
) -> Result<(), RimeError> {
    run_blocking(move || write_config_file_content_sync(filename, content)).await
}
