use crate::backend::*;
use crate::*;
use serde_yaml::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::{OnceLock, RwLock};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

type DictCacheMap = HashMap<PathBuf, (u64, u64, usize)>;
static DICT_ENTRY_CACHE: OnceLock<RwLock<DictCacheMap>> = OnceLock::new();

fn get_dict_entry_cache() -> &'static RwLock<DictCacheMap> {
    DICT_ENTRY_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub(crate) fn count_dict_entries_stream(
    path: &Path,
    size_bytes: u64,
    modified: Option<u64>,
) -> usize {
    let mod_val = modified.unwrap_or(0);
    // Check cache
    if let Ok(cache) = get_dict_entry_cache().read() {
        if let Some(&(cached_size, cached_mod, cached_count)) = cache.get(path) {
            if cached_size == size_bytes && cached_mod == mod_val {
                return cached_count;
            }
        }
    }

    // Stream lines with BufReader
    let Ok(file) = fs::File::open(path) else {
        return 0;
    };
    let reader = BufReader::new(file);
    let mut entry_count = 0usize;
    let mut past_header = false;

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed == "..." {
            past_header = true;
            continue;
        }
        if !past_header {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.contains('\t') {
            entry_count += 1;
        }
    }

    // Update cache
    if let Ok(mut cache) = get_dict_entry_cache().write() {
        cache.insert(path.to_path_buf(), (size_bytes, mod_val, entry_count));
    }

    entry_count
}

pub(crate) fn list_dictionaries_sync() -> Result<Vec<DictInfo>, RimeError> {
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(Vec::new());
    }

    let mut dicts = Vec::new();
    let mut pending_dirs = vec![user_dir.clone()];

    while let Some(dir) = pending_dirs.pop() {
        if should_skip_dict_directory(&dir, &user_dir) {
            continue;
        }

        let entries = fs::read_dir(&dir)
            .map_err(|err| RimeError::FileOperationError(format!("读取 Rime 目录失败: {err}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|err| RimeError::FileOperationError(format!("检查文件失败: {err}")))?;
            let path = entry.path();
            if path.is_dir() {
                pending_dirs.push(path);
                continue;
            }

            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };

            if !name.ends_with(".dict.yaml") {
                continue;
            }

            let metadata = entry.metadata().ok();
            let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata
                .and_then(|m| m.modified().ok())
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());

            let entry_count = count_dict_entries_stream(&path, size_bytes, modified);

            let display_name = path
                .strip_prefix(&user_dir)
                .ok()
                .map(|relative| relative.display().to_string().replace('\\', "/"))
                .unwrap_or_else(|| name.to_string());

            dicts.push(DictInfo {
                name: display_name,
                path: path.display().to_string(),
                entry_count,
                size_bytes,
                modified,
            });
        }
    }

    dicts.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(dicts)
}

pub(crate) fn should_skip_dict_directory(dir: &Path, user_dir: &Path) -> bool {
    if dir == user_dir {
        return false;
    }
    let Some(name) = dir.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    name.eq_ignore_ascii_case("build")
        || name.eq_ignore_ascii_case("sync")
        || name.eq_ignore_ascii_case("trash")
        || name.eq_ignore_ascii_case("opencc")
        || name.starts_with("backup-")
        || name.ends_with(".userdb")
        || name == ".git"
}

pub(crate) const MAX_DICT_HEALTH_BYTES: u64 = 20 * 1024 * 1024;

pub(crate) fn analyze_dict_health(path: &Path) -> Option<DictHealth> {
    let metadata = fs::metadata(path).ok()?;
    let truncated = metadata.len() > MAX_DICT_HEALTH_BYTES;
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut entries = 0usize;
    let mut duplicate_exact_lines = 0usize;
    let mut long_low_weight_entries = 0usize;
    let mut seen = std::collections::HashSet::new();
    let mut past_header = false;
    let mut bytes_seen = 0u64;

    for line in reader.lines().map_while(Result::ok) {
        bytes_seen = bytes_seen.saturating_add((line.len() + 1) as u64);
        if truncated && bytes_seen > MAX_DICT_HEALTH_BYTES {
            break;
        }

        let trimmed = line.trim();
        if trimmed == "..." {
            past_header = true;
            continue;
        }
        if !past_header && (trimmed.starts_with("name:") || trimmed.starts_with("---")) {
            continue;
        }
        if !is_dictionary_entry_line(trimmed) {
            continue;
        }

        entries += 1;
        if !seen.insert(trimmed.to_string()) {
            duplicate_exact_lines += 1;
        }

        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.len() >= 3 && parts[0].chars().count() > 12 && parts.last() == Some(&"1") {
            long_low_weight_entries += 1;
        }
    }

    Some(DictHealth {
        entries,
        duplicate_exact_lines,
        long_low_weight_entries,
        truncated,
    })
}

pub(crate) fn validate_dictionary_path(
    user_dir: &Path,
    dict_name: &str,
) -> Result<PathBuf, RimeError> {
    if !dict_name.ends_with(".dict.yaml") {
        return Err(RimeError::InvalidDictionaryPath(
            "只能操作 .dict.yaml 词库文件".to_string(),
        ));
    }

    let relative = PathBuf::from(dict_name.replace('\\', "/"));
    if relative.components().any(|component| {
        matches!(
            component,
            std::path::Component::Prefix(_)
                | std::path::Component::RootDir
                | std::path::Component::ParentDir
        )
    }) {
        return Err(RimeError::InvalidDictionaryPath("词库路径无效".to_string()));
    }

    let path = user_dir.join(relative);
    if !path.exists() || !path.is_file() {
        return Err(RimeError::DictionaryNotFound("词库文件不存在".to_string()));
    }

    Ok(path)
}

pub(crate) fn dictionary_reference_from_name(name: &str) -> String {
    name.trim_end_matches(".dict.yaml").replace('\\', "/")
}

pub(crate) fn dictionary_file_name_from_reference(reference: &str) -> String {
    format!("{}.dict.yaml", reference.trim_end_matches(".dict.yaml"))
}

pub(crate) fn parse_import_tables(contents: &str) -> Vec<String> {
    if let Some(Value::Sequence(items)) = yaml_lookup(contents, "import_tables") {
        let imports = items
            .iter()
            .filter_map(yaml_value_to_string)
            .map(|value| {
                value
                    .trim()
                    .trim_end_matches(".dict.yaml")
                    .replace('\\', "/")
            })
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !imports.is_empty() {
            return imports;
        }
    }

    let mut imports = Vec::new();
    let mut in_import_tables = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import_tables:") || trimmed.starts_with("\"import_tables\":") {
            in_import_tables = true;
            continue;
        }
        if !in_import_tables {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("- ") {
            let value = value
                .split('#')
                .next()
                .unwrap_or(value)
                .trim()
                .trim_matches('"')
                .trim_end_matches(".dict.yaml")
                .replace('\\', "/");
            if !value.is_empty() {
                imports.push(value);
            }
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
    }
    imports
}

pub(crate) fn resolve_schema_path(user_dir: &Path, schema_id: &str) -> Option<PathBuf> {
    let user_schema = user_dir.join(format!("{schema_id}.schema.yaml"));
    if user_schema.exists() {
        return Some(user_schema);
    }

    locate_deployer()
        .and_then(|d| d.parent().map(|p| p.join("data")))
        .into_iter()
        .chain(std::iter::once(PathBuf::from(
            r"C:\Program Files\Rime\weasel-0.17.4\data",
        )))
        .chain(std::iter::once(PathBuf::from(
            r"C:\Program Files (x86)\Rime\weasel-0.17.4\data",
        )))
        .map(|dir| dir.join(format!("{schema_id}.schema.yaml")))
        .find(|path| path.exists())
}

pub(crate) fn current_schema_dictionary(
    user_dir: &Path,
) -> (Option<String>, Option<String>, Option<String>) {
    let schema_id = parse_schema(&read_to_string(&user_dir.join("default.custom.yaml")));
    let Some(schema_id_value) = schema_id.as_deref() else {
        return (schema_id, None, None);
    };

    let schema_path = resolve_schema_path(user_dir, schema_id_value);
    let schema_contents = schema_path
        .as_deref()
        .map(read_to_string)
        .unwrap_or_default();
    let schema_name = parse_quoted_value(&schema_contents, "schema/name")
        .or_else(|| parse_string_after_key(&schema_contents, "name:"));
    let dictionary = parse_string_after_key(&schema_contents, "translator/dictionary")
        .or_else(|| parse_string_after_key(&schema_contents, "dictionary:"))
        .or_else(|| Some(schema_id_value.to_string()));

    (schema_id, schema_name, dictionary)
}

pub(crate) fn dict_info_to_reference(info: &DictInfo) -> String {
    dictionary_reference_from_name(&info.name)
}

pub(crate) fn read_dictionary_config_sync() -> Result<DictionaryConfig, RimeError> {
    let user_dir = rime_user_dir()?;
    let dictionaries = list_dictionaries_sync()?;
    let (schema_id, schema_name, main_dictionary) = current_schema_dictionary(&user_dir);

    let Some(main_dictionary_value) = main_dictionary.clone() else {
        return Ok(DictionaryConfig {
            schema_id,
            schema_name,
            main_dictionary: None,
            main_dictionary_path: None,
            enabled: Vec::new(),
            available: dictionaries,
            missing: Vec::new(),
        });
    };

    let main_path = user_dir.join(dictionary_file_name_from_reference(&main_dictionary_value));
    let imports = parse_import_tables(&read_to_string(&main_path));
    let dict_by_ref = dictionaries
        .iter()
        .map(|dict| (dict_info_to_reference(dict), dict))
        .collect::<std::collections::HashMap<_, _>>();

    let mut enabled = Vec::new();
    let mut missing = Vec::new();
    for reference in imports {
        if let Some(dict) = dict_by_ref.get(&reference) {
            enabled.push(DictionaryReference {
                reference,
                path: Some(dict.path.clone()),
                exists: true,
                entry_count: Some(dict.entry_count),
                size_bytes: Some(dict.size_bytes),
            });
        } else {
            missing.push(DictionaryReference {
                reference,
                path: None,
                exists: false,
                entry_count: None,
                size_bytes: None,
            });
        }
    }

    let enabled_refs = enabled
        .iter()
        .map(|entry| entry.reference.clone())
        .collect::<std::collections::HashSet<_>>();
    let available = dictionaries
        .into_iter()
        .filter(|dict| dict_info_to_reference(dict) != main_dictionary_value)
        .filter(|dict| !enabled_refs.contains(&dict_info_to_reference(dict)))
        .collect();

    Ok(DictionaryConfig {
        schema_id,
        schema_name,
        main_dictionary: Some(main_dictionary_value),
        main_dictionary_path: if main_path.exists() {
            Some(main_path.display().to_string())
        } else {
            None
        },
        enabled,
        available,
        missing,
    })
}

pub(crate) fn render_main_dictionary(dictionary_id: &str, imports: &[String]) -> String {
    let mut contents = vec![
        "---".to_string(),
        format!("name: {dictionary_id}"),
        format!("version: \"{}\"", timestamp()),
        "sort: by_weight".to_string(),
        "import_tables:".to_string(),
    ];
    for reference in imports {
        contents.push(format!("  - {reference}"));
    }
    contents.push("...".to_string());
    contents.push(String::new());
    contents.join("\n")
}

pub(crate) fn save_dictionary_imports_sync(
    imports: Vec<String>,
) -> Result<DictionaryConfig, RimeError> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let (_, _, main_dictionary) = current_schema_dictionary(&user_dir);
    let main_dictionary = main_dictionary
        .ok_or_else(|| RimeError::ConfigNotFound("当前方案未找到主词库".to_string()))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    let mut seen = std::collections::HashSet::new();
    let cleaned = imports
        .into_iter()
        .map(|reference| {
            reference
                .trim()
                .trim_end_matches(".dict.yaml")
                .replace('\\', "/")
        })
        .filter(|reference| !reference.is_empty())
        .filter(|reference| !reference.contains("..") && !reference.starts_with('/'))
        .filter(|reference| seen.insert(reference.clone()))
        .collect::<Vec<_>>();

    let path = user_dir.join(dictionary_file_name_from_reference(&main_dictionary));
    write_text_file(
        &path,
        &render_main_dictionary(&main_dictionary, &cleaned),
        "写入主词库配置失败",
    )?;

    read_dictionary_config_sync()
}

pub(crate) fn add_dictionary_to_current_schema_sync(
    reference: String,
) -> Result<DictionaryConfig, RimeError> {
    let config = read_dictionary_config_sync()?;
    let reference = reference
        .trim()
        .trim_end_matches(".dict.yaml")
        .replace('\\', "/");
    if reference.is_empty() {
        return Err(RimeError::InvalidDictionaryPath(
            "词库引用不能为空".to_string(),
        ));
    }

    let mut imports = config
        .enabled
        .iter()
        .map(|entry| entry.reference.clone())
        .chain(config.missing.iter().map(|entry| entry.reference.clone()))
        .collect::<Vec<_>>();
    if !imports.iter().any(|item| item == &reference) {
        imports.push(reference);
    }
    save_dictionary_imports_sync(imports)
}

pub(crate) fn remove_dictionary_from_current_schema_sync(
    reference: String,
) -> Result<DictionaryConfig, RimeError> {
    let config = read_dictionary_config_sync()?;
    let reference = reference
        .trim()
        .trim_end_matches(".dict.yaml")
        .replace('\\', "/");
    let imports = config
        .enabled
        .iter()
        .map(|entry| entry.reference.clone())
        .chain(config.missing.iter().map(|entry| entry.reference.clone()))
        .filter(|item| item != &reference)
        .collect::<Vec<_>>();
    save_dictionary_imports_sync(imports)
}

pub(crate) fn get_dict_health_sync(dict_name: String) -> Result<DictHealth, RimeError> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;

    analyze_dict_health(&path)
        .ok_or_else(|| RimeError::DictionaryNotFound("词库分析失败".to_string()))
}

pub(crate) fn remove_duplicate_dictionary_lines(contents: &str) -> (String, usize) {
    let mut seen = std::collections::HashSet::new();
    let mut removed = 0usize;
    let mut lines = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if is_dictionary_entry_line(trimmed) && !seen.insert(trimmed.to_string()) {
            removed += 1;
            continue;
        }
        lines.push(line);
    }

    let mut cleaned = lines.join("\n");
    if contents.ends_with('\n') {
        cleaned.push('\n');
    }
    (cleaned, removed)
}

pub(crate) fn clean_dictionary_duplicates_sync(
    dict_name: String,
) -> Result<DictionaryCleanResult, RimeError> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;
    let contents = fs::read_to_string(&path)
        .map_err(|err| RimeError::FileOperationError(format!("读取词库失败: {err}")))?;
    let (cleaned, removed_duplicate_lines) = remove_duplicate_dictionary_lines(&contents);

    let backup_dir = if removed_duplicate_lines > 0 {
        let backup_dir = backup_user_config(&user_dir, BackupKind::BeforeSave)?;
        write_text_file(&path, &cleaned, "写入去重后的词库失败")?;
        Some(backup_dir.display().to_string())
    } else {
        None
    };

    let entries_after = analyze_dict_health(&path)
        .map(|health| health.entries)
        .unwrap_or_default();

    Ok(DictionaryCleanResult {
        name: dict_name,
        path: path.display().to_string(),
        removed_duplicate_lines,
        entries_after,
        backup_dir,
    })
}

pub(crate) fn sanitize_dict_id(source_name: &str) -> String {
    let stem = Path::new(source_name)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("imported");
    let mut id = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            id.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '.' {
            id.push('_');
        }
    }
    let id = id.trim_matches('_').replace('-', "_");
    if id.is_empty() {
        "imported".to_string()
    } else {
        id
    }
}

pub(crate) fn sanitize_dict_file_name(source_name: &str) -> String {
    let id = sanitize_dict_id(source_name);
    if id.ends_with(".dict") {
        format!("{id}.yaml")
    } else if id.ends_with("_dict") {
        format!("{}.yaml", id.replace("_dict", ".dict"))
    } else {
        format!("{id}.dict.yaml")
    }
}
