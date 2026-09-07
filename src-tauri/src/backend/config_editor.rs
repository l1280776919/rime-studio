use crate::backend::*;
use crate::*;
use std::fs;
use std::path::{Path, PathBuf};

const EDITABLE_EXTENSIONS: &[&str] = &[".yaml", ".yml", ".txt", ".lua"];

pub(crate) fn is_editable_config_name(name: &str) -> bool {
    EDITABLE_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
}

pub(crate) fn validate_config_relpath(filename: &str) -> Result<(), RimeError> {
    let invalid = filename.is_empty()
        || filename.contains('\\')
        || filename.contains("..")
        || filename.starts_with('/')
        || filename.chars().nth(1) == Some(':');
    if invalid || !is_editable_config_name(filename) {
        return Err(RimeError::ConfigNotFound(
            "只能访问 Rime 用户目录中的 YAML、TXT 或 Lua 文件".to_string(),
        ));
    }

    Ok(())
}

fn validate_config_content(filename: &str, content: &str) -> Result<(), RimeError> {
    if filename.ends_with(".yaml") || filename.ends_with(".yml") {
        serde_yaml::from_str::<serde_yaml::Value>(content).map_err(|err| {
            let location = err
                .location()
                .map(|location| format!("第 {} 行，第 {} 列", location.line(), location.column()))
                .unwrap_or_else(|| "未知位置".to_string());
            RimeError::YamlParseError(format!("{location}: {err}"))
        })?;
    }
    Ok(())
}

fn should_skip_config_directory(dir: &Path, user_dir: &Path) -> bool {
    if dir == user_dir {
        return false;
    }
    let Some(name) = dir.file_name().and_then(|value| value.to_str()) else {
        return true;
    };
    name.eq_ignore_ascii_case("build")
        || name.eq_ignore_ascii_case("sync")
        || name.ends_with(".userdb")
        || name.starts_with("backup-")
        || name.starts_with('.')
}

fn collect_config_files(user_dir: &Path, dir: &Path, files: &mut Vec<FileStatus>) {
    if should_skip_config_directory(dir, user_dir) {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_config_files(user_dir, &path, files);
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !is_editable_config_name(name) {
            continue;
        }
        let rel = path
            .strip_prefix(user_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        files.push(file_status(user_dir, &rel));
    }
}

fn resolve_config_path(filename: &str, must_exist: bool) -> Result<PathBuf, RimeError> {
    validate_config_relpath(filename)?;
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        if must_exist {
            return Err(RimeError::ConfigNotFound(format!(
                "配置文件不存在: {filename}"
            )));
        }
        return Ok(join_user_rel(&user_dir, filename));
    }

    let user_dir = fs::canonicalize(&user_dir).unwrap_or(user_dir);
    let path = join_user_rel(&user_dir, filename);

    if must_exist {
        let canonical = fs::canonicalize(&path)
            .map_err(|_| RimeError::ConfigNotFound(format!("配置文件不存在: {filename}")))?;
        if !canonical.starts_with(&user_dir) {
            return Err(RimeError::ConfigNotFound(
                "只能访问 Rime 用户目录中的 YAML、TXT 或 Lua 文件".to_string(),
            ));
        }
        return Ok(canonical);
    }

    if let Some(parent) = path.parent() {
        if parent.exists() {
            let parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            if !parent.starts_with(&user_dir) {
                return Err(RimeError::ConfigNotFound(
                    "只能访问 Rime 用户目录中的 YAML、TXT 或 Lua 文件".to_string(),
                ));
            }
        }
    }

    Ok(path)
}

pub(crate) fn list_yaml_config_files_sync() -> Result<Vec<FileStatus>, RimeError> {
    let user_dir = rime_user_dir()?;
    let mut files = Vec::new();
    if user_dir.exists() {
        collect_config_files(&user_dir, &user_dir, &mut files);
    }
    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(files)
}

pub(crate) fn read_config_file_content_sync(filename: String) -> Result<String, RimeError> {
    let path = resolve_config_path(&filename, true)?;
    if !path.is_file() {
        return Err(RimeError::ConfigNotFound(format!(
            "配置文件不存在: {filename}"
        )));
    }

    fs::read_to_string(&path)
        .map_err(|err| RimeError::FileOperationError(format!("读取文件失败: {err}")))
}

pub(crate) fn write_config_file_content_sync(
    filename: String,
    content: String,
) -> Result<(), RimeError> {
    validate_config_content(&filename, &content)?;
    let path = resolve_config_path(&filename, false)?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::SettingsError(format!("创建 Rime 目录失败: {err}")))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RimeError::FileOperationError(format!("创建目录失败: {err}")))?;
    }

    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(&path, &content, "写入配置文件失败")?;
    Ok(())
}
