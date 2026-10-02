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
    expected: FileRevision,
) -> Result<bool, RimeError> {
    run_blocking(move || {
        write_config_file_guarded_sync(filename, content, expected)?;
        Ok(true)
    })
    .await
}

/// Read value and revision together; absence is meaningful during conflict recovery.
#[tauri::command]
pub(crate) async fn read_config_file_revision(filename: String) -> Result<FileRevision, RimeError> {
    run_blocking(move || read_file_revision(&resolve_config_path(&filename, false)?)).await
}

/// Reuse the same line diff as configuration and backup previews.
#[tauri::command]
pub(crate) async fn compare_config_text(
    before: String,
    after: String,
) -> Result<Vec<String>, RimeError> {
    run_blocking(move || {
        if before.len() + after.len() > 2 * 1024 * 1024 {
            return Ok(vec!["内容超过 2 MiB，已省略差异计算，请展开原文比较".into()]);
        }
        Ok(build_text_diff(&before, &after))
    })
    .await
}
