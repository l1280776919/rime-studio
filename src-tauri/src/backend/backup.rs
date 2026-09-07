use crate::backend::*;
use crate::*;
use std::{
    ffi::OsStr,
    fs,
    io::{self},
    path::{Path, PathBuf},
    process::{self},
};

pub(crate) fn is_dictionary_entry_line(trimmed: &str) -> bool {
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && trimmed != "---"
        && trimmed != "..."
        && !trimmed.starts_with("name:")
        && !trimmed.starts_with("version:")
        && !trimmed.starts_with("sort:")
        && trimmed.contains('\t')
}

pub(crate) fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "unknown-time".to_string())
}

pub(crate) fn copy_if_exists(source: &Path, target: &Path) -> io::Result<()> {
    if source.exists() {
        fs::copy(source, target)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) enum BackupKind {
    Manual,
    BeforeSave,
    BeforeRestore,
    BeforeInstall,
}

impl BackupKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            BackupKind::Manual => "manual",
            BackupKind::BeforeSave => "before-save",
            BackupKind::BeforeRestore => "before-restore",
            BackupKind::BeforeInstall => "before-install",
        }
    }
}

pub(crate) fn backup_kind_from_name(name: &str) -> String {
    let marker = "backup-rime-studio-";
    let Some(rest) = name.strip_prefix(marker) else {
        return BackupKind::Manual.as_str().to_string();
    };

    if rest.starts_with("before-save-") {
        BackupKind::BeforeSave.as_str().to_string()
    } else if rest.starts_with("before-restore-") {
        BackupKind::BeforeRestore.as_str().to_string()
    } else if rest.starts_with("before-install-") {
        BackupKind::BeforeInstall.as_str().to_string()
    } else {
        BackupKind::Manual.as_str().to_string()
    }
}

pub(crate) fn create_unique_backup_dir(
    backup_root: &Path,
    kind: BackupKind,
) -> Result<PathBuf, RimeError> {
    for suffix in 0..100 {
        let base = format!("backup-rime-studio-{}-{}", kind.as_str(), timestamp());
        let name = if suffix == 0 {
            base
        } else {
            format!("{base}-{suffix}")
        };
        let path = backup_root.join(name);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|err| RimeError::BackupError(format!("创建备份目录失败: {err}")))?;
            return Ok(path);
        }
    }

    Err(RimeError::BackupError(
        "创建备份目录失败: 无法生成唯一目录名".to_string(),
    ))
}

pub(crate) fn is_auto_backup_kind(kind: &str) -> bool {
    matches!(kind, "before-save" | "before-restore" | "before-install")
}

pub(crate) fn backup_dir_modified(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

pub(crate) fn prune_old_auto_backups(
    backup_root: &Path,
    keep_limit: usize,
) -> Result<usize, RimeError> {
    if !backup_root.exists() {
        return Ok(0);
    }

    let mut auto_backups = Vec::new();
    for entry in fs::read_dir(backup_root)
        .map_err(|err| RimeError::BackupError(format!("读取备份目录失败: {err}")))?
    {
        let entry =
            entry.map_err(|err| RimeError::BackupError(format!("检查备份目录失败: {err}")))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("backup-rime-studio-") {
            continue;
        }
        if !is_auto_backup_kind(&backup_kind_from_name(name)) {
            continue;
        }

        auto_backups.push((backup_dir_modified(&path), name.to_string(), path));
    }

    auto_backups.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    let mut removed = 0usize;
    for (_, _, path) in auto_backups.into_iter().skip(keep_limit) {
        fs::remove_dir_all(&path)
            .map_err(|err| RimeError::BackupError(format!("清理旧自动备份失败: {err}")))?;
        removed += 1;
    }

    Ok(removed)
}

pub(crate) fn write_text_file(path: &Path, contents: &str, context: &str) -> Result<(), RimeError> {
    let parent = path
        .parent()
        .ok_or_else(|| RimeError::FileOperationError(format!("{context}: 目标路径无效")))?;
    fs::create_dir_all(parent)
        .map_err(|err| RimeError::FileOperationError(format!("{context}: 创建目录失败: {err}")))?;

    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| RimeError::FileOperationError(format!("{context}: 文件名无效")))?;
    let temp_path = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        process::id(),
        timestamp()
    ));

    fs::write(&temp_path, contents)
        .map_err(|err| RimeError::FileOperationError(format!("{context}: {err}")))?;
    if let Err(rename_err) = fs::rename(&temp_path, path) {
        if path.exists() {
            fs::remove_file(path).map_err(|remove_err| {
                let _ = fs::remove_file(&temp_path);
                RimeError::FileOperationError(format!("{context}: 替换旧文件失败: {remove_err}"))
            })?;
            fs::rename(&temp_path, path).map_err(|retry_err| {
                let _ = fs::remove_file(&temp_path);
                RimeError::FileOperationError(format!("{context}: {retry_err}"))
            })?;
        } else {
            let _ = fs::remove_file(&temp_path);
            return Err(RimeError::FileOperationError(format!(
                "{context}: {rename_err}"
            )));
        }
    }

    Ok(())
}

pub(crate) fn is_managed_config_file(name: &str) -> bool {
    name.ends_with(".custom.yaml")
        || name.ends_with(".dict.yaml")
        || name == "custom_phrase.txt"
        || name == "default.yaml"
        || name == "weasel.yaml"
}

pub(crate) fn is_manual_backup_file(name: &str) -> bool {
    is_managed_config_file(name)
        || name.ends_with(".schema.yaml")
        || name.ends_with(".lua")
        || name == "installation.yaml"
        || name == "user.yaml"
}

pub(crate) fn backup_scope_label(kind: &str) -> String {
    if kind == "manual" {
        "配置快照：*.custom.yaml、词库、短语、方案、Lua、installation.yaml。不含 build/、sync/、*.userdb。"
            .to_string()
    } else {
        "配置快照：*.custom.yaml、词库、短语。不含方案源文件、Lua、用户词库和 build/。".to_string()
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct BackupMeta {
    note: Option<String>,
    kind: String,
    created_at: u64,
}

fn write_backup_meta(dir: &Path, kind: BackupKind, note: Option<&str>) -> Result<(), RimeError> {
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let meta = BackupMeta {
        note: note
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string),
        kind: kind.as_str().to_string(),
        created_at,
    };
    let payload = serde_json::to_string_pretty(&meta)
        .map_err(|err| RimeError::BackupError(format!("写入备份备注失败: {err}")))?;
    fs::write(dir.join("backup-meta.json"), payload)
        .map_err(|err| RimeError::BackupError(format!("写入备份备注失败: {err}")))
}

fn read_backup_meta(dir: &Path) -> BackupMeta {
    fs::read_to_string(dir.join("backup-meta.json"))
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

pub(crate) fn backup_user_config(user_dir: &Path, kind: BackupKind) -> Result<PathBuf, RimeError> {
    backup_user_config_with_note(user_dir, kind, None)
}

pub(crate) fn backup_user_config_with_note(
    user_dir: &Path,
    kind: BackupKind,
    note: Option<&str>,
) -> Result<PathBuf, RimeError> {
    let backup_root = app_data_dir()?;
    fs::create_dir_all(&backup_root)
        .map_err(|err| RimeError::BackupError(format!("创建备份根目录失败: {err}")))?;
    let backup_dir = create_unique_backup_dir(&backup_root, kind)?;

    for entry in fs::read_dir(user_dir)
        .map_err(|err| RimeError::BackupError(format!("读取 Rime 目录失败: {err}")))?
    {
        let entry =
            entry.map_err(|err| RimeError::BackupError(format!("检查 Rime 文件失败: {err}")))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        let include = if matches!(kind, BackupKind::Manual) {
            is_manual_backup_file(name)
        } else {
            is_managed_config_file(name)
        };
        if include {
            copy_if_exists(&path, &backup_dir.join(name))
                .map_err(|err| RimeError::BackupError(format!("备份 {name} 失败: {err}")))?;
        }
    }

    write_backup_meta(&backup_dir, kind, note)?;

    if !matches!(kind, BackupKind::Manual) {
        let _ = prune_old_auto_backups(&backup_root, AUTO_BACKUP_KEEP_LIMIT);
    }

    Ok(backup_dir)
}

pub(crate) fn list_backup_dirs(_user_dir: &Path) -> Result<Vec<BackupEntry>, RimeError> {
    let backup_root = app_data_dir()?;
    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&backup_root)
        .map_err(|err| RimeError::BackupError(format!("读取备份目录失败: {err}")))?
    {
        let entry =
            entry.map_err(|err| RimeError::BackupError(format!("检查 Rime 文件失败: {err}")))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("backup-rime-studio-") {
            continue;
        }

        let modified = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());
        let files = fs::read_dir(&path)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|item| {
                        item.path().is_file()
                            && item.file_name().to_string_lossy() != "backup-meta.json"
                    })
                    .count()
            })
            .unwrap_or(0);

        let kind = backup_kind_from_name(name);
        let meta = read_backup_meta(&path);
        backups.push(BackupEntry {
            name: name.to_string(),
            path: path.display().to_string(),
            kind: kind.clone(),
            modified,
            files,
            scope: backup_scope_label(&kind),
            note: meta.note,
        });
    }

    backups.sort_by_key(|right| std::cmp::Reverse(right.modified));
    Ok(backups)
}

pub(crate) fn validated_backup_dir(
    _user_dir: &Path,
    backup_name: &str,
) -> Result<PathBuf, RimeError> {
    if !backup_name.starts_with("backup-rime-studio-")
        || backup_name.contains('/')
        || backup_name.contains('\\')
        || backup_name.contains("..")
    {
        return Err(RimeError::BackupError("无效的备份名称".to_string()));
    }

    let backup_root = app_data_dir()?;
    let backup_dir = backup_root.join(backup_name);
    if !backup_dir.is_dir() {
        return Err(RimeError::BackupError(format!("备份不存在: {backup_name}")));
    }

    Ok(backup_dir)
}

pub(crate) fn restore_backup_dir(
    user_dir: &Path,
    backup_dir: &Path,
) -> Result<RestoreResult, RimeError> {
    let safety_backup_dir = backup_user_config(user_dir, BackupKind::BeforeRestore)?;
    let mut restored_files = 0usize;

    for entry in fs::read_dir(backup_dir)
        .map_err(|err| RimeError::BackupError(format!("读取备份失败: {err}")))?
    {
        let entry =
            entry.map_err(|err| RimeError::BackupError(format!("检查备份文件失败: {err}")))?;
        let source = entry.path();
        if !source.is_file() {
            continue;
        }

        let Some(name) = source.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if name == "backup-meta.json"
            || name.contains("..")
            || name.contains('/')
            || name.contains('\\')
        {
            continue;
        }

        fs::copy(&source, user_dir.join(name))
            .map_err(|err| RimeError::BackupError(format!("恢复 {name} 失败: {err}")))?;
        restored_files += 1;
    }

    Ok(RestoreResult {
        restored_files,
        safety_backup_dir: safety_backup_dir.display().to_string(),
    })
}
