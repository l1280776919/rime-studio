use crate::backend::*;
use crate::*;
use std::fs;

/// List all .yaml files in the Rime user directory.
/// Excludes backup directories (names starting with "backup-").
/// Returns files sorted by modification time (newest first) with FileStatus metadata.
pub(crate) fn list_yaml_config_files_sync() -> Result<Vec<FileStatus>, RimeError> {
    let user_dir = rime_user_dir()?;
    let mut files = Vec::new();

    let dir_entries = fs::read_dir(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("读取 Rime 目录失败: {err}")))?;

    for entry in dir_entries {
        let entry =
            entry.map_err(|err| RimeError::FileOperationError(format!("读取目录项失败: {err}")))?;
        let path = entry.path();

        // Skip directories and non-yaml files
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Only include .yaml files
        if !file_name.ends_with(".yaml") && !file_name.ends_with(".yml") {
            continue;
        }

        files.push(file_status(&user_dir, file_name));
    }

    // Sort by name
    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}

/// Read the full content of a config file by filename (relative to Rime user dir).
/// Validates that the filename does not contain path traversal characters.
pub(crate) fn read_config_file_content_sync(filename: String) -> Result<String, RimeError> {
    // Prevent path traversal
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(RimeError::ConfigNotFound("文件名包含非法字符".to_string()));
    }

    let path = rime_user_dir()?.join(&filename);
    if !path.exists() || !path.is_file() {
        return Err(RimeError::ConfigNotFound(format!(
            "配置文件不存在: {filename}"
        )));
    }

    fs::read_to_string(&path)
        .map_err(|err| RimeError::FileOperationError(format!("读取文件失败: {err}")))
}

/// Write content to a config file by filename (relative to Rime user dir).
/// Creates an auto-backup before overwriting.
pub(crate) fn write_config_file_content_sync(
    filename: String,
    content: String,
) -> Result<(), RimeError> {
    // Prevent path traversal
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(RimeError::ConfigNotFound("文件名包含非法字符".to_string()));
    }

    let user_dir = rime_user_dir()?;
    let path = user_dir.join(&filename);

    // Create user dir if it doesn't exist
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::SettingsError(format!("创建 Rime 目录失败: {err}")))?;

    // Auto-backup before writing
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    // Atomic write via temp file
    write_text_file(&path, &content, "写入配置文件失败")?;

    Ok(())
}
