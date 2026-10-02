//! Versioned ZIP format for portable text configuration. Archives are read into
//! bounded memory and never extracted using archive-provided filesystem paths.
use crate::backend::*;
use crate::types::RimeError;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    io::{Cursor, Read, Write},
};

pub(crate) const MIGRATION_MAX_ARCHIVE: usize = 64 * 1024 * 1024;
pub(crate) const MIGRATION_MAX_FILE: usize = 64 * 1024 * 1024;
pub(crate) const MIGRATION_MAX_TOTAL: usize = 256 * 1024 * 1024;
pub(crate) const MIGRATION_MAX_FILES: usize = 1000;
const MANIFEST: &str = "rime-studio-migration.json";

#[derive(Serialize, Deserialize)]
pub(crate) struct MigrationManifest {
    pub format: String,
    pub format_version: u32,
    pub app_version: String,
    pub created_at: String,
    pub files: Vec<MigrationFileInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MigrationFileInfo {
    pub name: String,
    pub category: String,
    pub bytes: usize,
}

pub(crate) fn migration_error(message: impl Into<String>) -> RimeError {
    RimeError::FileOperationError(message.into())
}

/// Classification also serves as the shared export/import allowlist. Machine
/// identity and live user databases must never be copied onto another device.
pub(crate) fn migration_category(name: &str) -> Option<&'static str> {
    if !valid_user_relative_path(name) || name.len() > 240 {
        return None;
    }
    let lower = name.to_lowercase();
    if lower.split('/').any(|part| {
        part.starts_with('.')
            || part.starts_with("backup-")
            || part.ends_with(".userdb")
            || part.ends_with(".userdb.txt")
            || matches!(part, "build" | "sync" | "installation.yaml")
    }) {
        return None;
    }
    if lower.ends_with(".schema.yaml") {
        Some("schemas")
    } else if lower.ends_with(".dict.yaml") {
        Some("dictionaries")
    } else if lower.ends_with(".lua") {
        Some("lua")
    } else if lower.rsplit('/').next() == Some("custom_phrase.txt") {
        Some("phrases")
    } else if is_editable_config_name(&lower) {
        Some("config")
    } else {
        None
    }
}

pub(crate) fn encode_migration_archive(
    files: &BTreeMap<String, String>,
) -> Result<Vec<u8>, RimeError> {
    let manifest = MigrationManifest {
        format: "rime-studio-migration".into(),
        format_version: 1,
        app_version: env!("CARGO_PKG_VERSION").into(),
        created_at: timestamp(),
        files: files
            .iter()
            .map(|(name, content)| MigrationFileInfo {
                name: name.clone(),
                category: migration_category(name).unwrap_or("config").into(),
                bytes: content.len(),
            })
            .collect(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    // Include the manifest in the same expanded-size budget used by the reader.
    let expanded_size = files.values().fold(manifest_bytes.len(), |total, content| {
        total.saturating_add(content.len())
    });
    if expanded_size > MIGRATION_MAX_TOTAL {
        return Err(migration_error(
            "迁移包解压内容超过大小限制，请减少导出类别",
        ));
    }
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    archive
        .start_file(MANIFEST, options)
        .map_err(|err| migration_error(err.to_string()))?;
    archive.write_all(&manifest_bytes)?;
    for (name, content) in files {
        archive
            .start_file(format!("files/{name}"), options)
            .map_err(|err| migration_error(err.to_string()))?;
        archive.write_all(content.as_bytes())?;
    }
    let data = archive
        .finish()
        .map_err(|err| migration_error(err.to_string()))?
        .into_inner();
    if data.len() > MIGRATION_MAX_ARCHIVE {
        return Err(migration_error("迁移包超过 64 MiB，请减少导出类别"));
    }
    Ok(data)
}

pub(crate) fn decode_migration_archive(
    data: Vec<u8>,
) -> Result<(MigrationManifest, BTreeMap<String, String>), RimeError> {
    if data.len() > MIGRATION_MAX_ARCHIVE {
        return Err(migration_error("迁移包不能超过 64 MiB"));
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(data))
        .map_err(|_| migration_error("无法读取 ZIP 迁移包"))?;
    if archive.len() > MIGRATION_MAX_FILES + 1 {
        return Err(migration_error("迁移包文件数量超过 1000"));
    }
    let mut entries = BTreeMap::new();
    let mut seen = HashSet::new();
    let mut total = 0usize;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| migration_error(err.to_string()))?;
        let name = entry.name().to_string();
        if entry.is_dir()
            || entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
            || !seen.insert(name.to_lowercase())
        {
            return Err(migration_error("迁移包含有目录、链接或重复文件"));
        }
        if name != MANIFEST
            && !name
                .strip_prefix("files/")
                .is_some_and(|path| migration_category(path).is_some())
        {
            return Err(migration_error(format!("迁移包含有不允许的路径：{name}")));
        }
        let limit = if name == MANIFEST {
            1024 * 1024
        } else {
            MIGRATION_MAX_FILE
        };
        if entry.size() > limit as u64 {
            return Err(migration_error("迁移包单文件超过大小限制"));
        }
        let mut bytes = Vec::new();
        (&mut entry)
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)?;
        total = total.saturating_add(bytes.len());
        if bytes.len() > limit || total > MIGRATION_MAX_TOTAL {
            return Err(migration_error("迁移包解压内容超过大小限制"));
        }
        let contents =
            String::from_utf8(bytes).map_err(|_| migration_error("迁移包配置必须是 UTF-8 文本"))?;
        entries.insert(name, contents);
    }
    let manifest: MigrationManifest = serde_json::from_str(
        &entries
            .remove(MANIFEST)
            .ok_or_else(|| migration_error("不是 Rime Studio 迁移包：缺少清单"))?,
    )?;
    if manifest.format != "rime-studio-migration" || manifest.format_version != 1 {
        return Err(migration_error("不支持此迁移包格式版本"));
    }
    if manifest.files.is_empty()
        || manifest.files.len() > MIGRATION_MAX_FILES
        || entries.len() != manifest.files.len()
    {
        return Err(migration_error("迁移包文件与清单不一致"));
    }
    let mut files = BTreeMap::new();
    let mut paths = HashSet::new();
    for info in &manifest.files {
        if migration_category(&info.name) != Some(info.category.as_str())
            || !paths.insert(info.name.to_lowercase())
        {
            return Err(migration_error("迁移清单含无效类别或重复路径"));
        }
        let contents = entries
            .remove(&format!("files/{}", info.name))
            .ok_or_else(|| migration_error("迁移清单中有缺失文件"))?;
        if contents.len() != info.bytes {
            return Err(migration_error("迁移文件长度与清单不符"));
        }
        files.insert(info.name.clone(), contents);
    }
    // Reject file/directory collisions before any target directories are created.
    for name in files.keys() {
        let mut parent = name.as_str();
        while let Some((prefix, _)) = parent.rsplit_once('/') {
            if paths.contains(&prefix.to_lowercase()) {
                return Err(migration_error("迁移包路径存在文件与目录冲突"));
            }
            parent = prefix;
        }
    }
    Ok((manifest, files))
}
