use crate::backend::*;
use crate::*;
use serde_yaml::Value;
use std::io::{Read, Write};
use std::time::Instant;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn list_dictionaries_sync() -> Result<Vec<DictInfo>, RimeError> {
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(Vec::new());
    }

    let mut dicts = Vec::new();
    let mut pending_dirs = vec![user_dir.clone()];

    while let Some(dir) = pending_dirs.pop() {
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

            let contents = fs::read_to_string(&path).unwrap_or_default();
            let mut entry_count = 0usize;
            let mut past_header = false;

            for line in contents.lines() {
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

    analyze_sogou(&path).ok_or_else(|| RimeError::DictionaryNotFound("词库分析失败".to_string()))
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

    let entries_after = analyze_sogou(&path)
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

pub(crate) fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub(crate) fn decode_utf16_le(data: &[u8]) -> String {
    let units = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
        .trim_matches(char::from(0))
        .trim()
        .to_string()
}

pub(crate) fn parse_scel_entries(data: &[u8]) -> Result<(Vec<DictionaryEntry>, usize), RimeError> {
    const PINYIN_TABLE_OFFSET: usize = 0x1540;

    if data.len() <= PINYIN_TABLE_OFFSET + 4 {
        return Err(RimeError::InvalidDictionaryPath(
            "搜狗 .scel 文件过小或格式不正确".to_string(),
        ));
    }

    let pinyin_count = read_u16_le(data, PINYIN_TABLE_OFFSET)
        .map(usize::from)
        .ok_or_else(|| RimeError::InvalidDictionaryPath("搜狗 .scel 拼音表损坏".to_string()))?;
    let mut pinyin_table = std::collections::HashMap::<u16, String>::new();
    let mut offset = PINYIN_TABLE_OFFSET + 4;
    for _ in 0..pinyin_count {
        let Some(index) = read_u16_le(data, offset) else {
            break;
        };
        let Some(byte_len) = read_u16_le(data, offset + 2).map(usize::from) else {
            break;
        };
        offset += 4;
        if byte_len == 0 || !byte_len.is_multiple_of(2) || offset + byte_len > data.len() {
            break;
        }

        let value = decode_utf16_le(&data[offset..offset + byte_len]);
        if !value.is_empty() {
            pinyin_table.insert(index, value);
        }
        offset += byte_len;
    }

    let mut entries = Vec::new();
    let mut skipped = 0usize;

    while offset + 4 <= data.len() {
        let Some(same_pinyin_count) = read_u16_le(data, offset).map(usize::from) else {
            break;
        };
        let Some(pinyin_byte_len) = read_u16_le(data, offset + 2).map(usize::from) else {
            break;
        };
        offset += 4;
        if same_pinyin_count == 0
            || pinyin_byte_len == 0
            || !pinyin_byte_len.is_multiple_of(2)
            || offset + pinyin_byte_len > data.len()
        {
            break;
        }

        let pinyin_indexes = data[offset..offset + pinyin_byte_len]
            .chunks_exact(2)
            .filter_map(|chunk| {
                let index = u16::from_le_bytes([chunk[0], chunk[1]]);
                pinyin_table.get(&index).cloned()
            })
            .collect::<Vec<_>>();
        let code = pinyin_indexes.join(" ");
        offset += pinyin_byte_len;

        for _ in 0..same_pinyin_count {
            if offset + 4 > data.len() {
                skipped += 1;
                break;
            }

            let word_byte_len = read_u16_le(data, offset).map(usize::from).unwrap_or(0);
            offset += 2;
            if word_byte_len == 0 || offset + word_byte_len > data.len() {
                skipped += 1;
                break;
            }
            let word = decode_utf16_le(&data[offset..offset + word_byte_len]);
            offset += word_byte_len;

            let ext_len = read_u16_le(data, offset).map(usize::from).unwrap_or(0);
            offset += 2;
            if offset + ext_len > data.len() {
                skipped += 1;
                break;
            }
            offset += ext_len;

            if word.is_empty() {
                skipped += 1;
            } else {
                entries.push((word, code.clone(), 1));
            }
        }
    }

    if entries.is_empty() {
        Err(RimeError::InvalidDictionaryPath(
            "未能从搜狗 .scel 文件中解析出词条".to_string(),
        ))
    } else {
        Ok((entries, skipped))
    }
}

pub(crate) fn is_sogou_bin_word(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= 80
        && value.chars().all(|ch| {
            let code = ch as u32;
            (0x4e00..=0x9fff).contains(&code)
                || (0x3400..=0x4dbf).contains(&code)
                || (0x20..=0x7e).contains(&code)
        })
}

pub(crate) fn sogou_bin_code_from_indexes(index_bytes: &[u8]) -> Option<String> {
    let mut syllables = Vec::new();
    for chunk in index_bytes.chunks_exact(2) {
        let index = u16::from_le_bytes([chunk[0], chunk[1]]) as usize;
        let syllable = SOGOU_BIN_PINYIN.get(index).copied().unwrap_or_default();
        if syllable.is_empty() {
            return None;
        }
        syllables.push(syllable);
    }

    if syllables.is_empty() {
        None
    } else {
        Some(syllables.join(" "))
    }
}

pub(crate) fn parse_sogou_bin_entries(
    data: &[u8],
) -> Result<(Vec<DictionaryEntry>, usize), RimeError> {
    if !data.starts_with(b"SGPU") {
        return Err(RimeError::InvalidDictionaryPath(
            "不是支持的搜狗用户词库 .bin 备份文件".to_string(),
        ));
    }

    let mut weighted = std::collections::BTreeMap::<(String, String), i32>::new();
    let mut skipped = 0usize;
    let mut offset = 0usize;

    while offset + 20 < data.len() {
        if data.get(offset..offset + 3) != Some(&[0, 1, 0]) {
            offset += 1;
            continue;
        }

        let record_kind = data[offset + 3];
        if record_kind != 2 && record_kind != 3 {
            offset += 1;
            continue;
        }

        let index_len = read_u16_le(data, offset + 4).map(usize::from).unwrap_or(0);
        if index_len == 0 || !index_len.is_multiple_of(2) || index_len > 80 {
            offset += 1;
            continue;
        }

        let index_offset = offset + 6;
        let meta_offset = index_offset + index_len;
        let word_len = read_u16_le(data, meta_offset + 2)
            .map(usize::from)
            .unwrap_or(0);
        let word_offset = meta_offset + 4;
        if word_len == 0
            || !word_len.is_multiple_of(2)
            || word_len > 160
            || word_offset + word_len + 2 + index_len > data.len()
        {
            offset += 1;
            continue;
        }

        let repeated_word_len = read_u16_le(data, word_offset + word_len)
            .map(usize::from)
            .unwrap_or(0);
        if repeated_word_len != word_len {
            offset += 1;
            continue;
        }

        let word = decode_utf16_le(&data[word_offset..word_offset + word_len]);
        if !is_sogou_bin_word(&word) {
            skipped += 1;
            offset += 1;
            continue;
        }

        let Some(code) = sogou_bin_code_from_indexes(&data[index_offset..index_offset + index_len])
        else {
            skipped += 1;
            offset = word_offset + word_len + 2 + index_len;
            continue;
        };

        *weighted.entry((word, code)).or_insert(0) += 1;
        offset = word_offset + word_len + 2 + index_len;
    }

    if weighted.is_empty() {
        return Err(RimeError::InvalidDictionaryPath(
            "未能从搜狗 .bin 备份中解析出可导入词条".to_string(),
        ));
    }

    let entries = weighted
        .into_iter()
        .map(|((word, code), weight)| (word, code, weight))
        .collect();
    Ok((entries, skipped))
}

pub(crate) fn parse_text_dictionary_entries(contents: &str) -> (Vec<DictionaryEntry>, usize) {
    let mut entries = Vec::new();
    let mut skipped = 0usize;
    let mut past_header = !contents.lines().any(|line| line.trim() == "...");

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "..." {
            past_header = true;
            continue;
        }
        if !past_header || trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" {
            continue;
        }

        let parts = trimmed.split('\t').map(str::trim).collect::<Vec<_>>();
        if parts.is_empty() || parts[0].is_empty() {
            skipped += 1;
            continue;
        }

        let text = parts[0].to_string();
        let code = parts.get(1).copied().unwrap_or_default().to_string();
        let weight = parts
            .get(2)
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(1);
        entries.push((text, code, weight));
    }

    (entries, skipped)
}

pub(crate) fn render_rime_dictionary(dict_id: &str, entries: &[DictionaryEntry]) -> String {
    let mut contents = vec![
        "# Imported by Rime Studio.".to_string(),
        "---".to_string(),
        format!("name: {dict_id}"),
        format!("version: \"{}\"", timestamp()),
        "sort: by_weight".to_string(),
        "...".to_string(),
    ];

    for (text, code, weight) in entries {
        contents.push(format!("{text}\t{code}\t{weight}"));
    }
    contents.push(String::new());
    contents.join("\n")
}

pub(crate) fn parse_dictionary_import_payload(
    source_name: String,
    data: Vec<u8>,
) -> Result<(String, String, Vec<DictionaryEntry>, usize, String), RimeError> {
    if data.is_empty() {
        return Err(RimeError::InvalidDictionaryPath("导入文件为空".to_string()));
    }
    if data.len() > 64 * 1024 * 1024 {
        return Err(RimeError::InvalidDictionaryPath(
            "导入文件超过 64MB".to_string(),
        ));
    }

    let dict_name = sanitize_dict_file_name(&source_name);
    let dict_id = dict_name.trim_end_matches(".dict.yaml");
    let lower_name = source_name.to_lowercase();
    let (entries, skipped_entries, rendered_contents) = if lower_name.ends_with(".scel") {
        let (entries, skipped) = parse_scel_entries(&data)?;
        let rendered = render_rime_dictionary(dict_id, &entries);
        (entries, skipped, rendered)
    } else if lower_name.ends_with(".bin") {
        let (entries, skipped) = parse_sogou_bin_entries(&data)?;
        let rendered = render_rime_dictionary(dict_id, &entries);
        (entries, skipped, rendered)
    } else {
        let contents = String::from_utf8(data).map_err(|_| {
            RimeError::InvalidDictionaryPath(
                "文本词库需要使用 UTF-8 编码；搜狗二进制词库请导入 .scel 文件".to_string(),
            )
        })?;
        let (entries, skipped) = parse_text_dictionary_entries(&contents);
        let rendered = if lower_name.ends_with(".dict.yaml")
            && contents.lines().any(|line| line.trim() == "...")
        {
            contents
        } else {
            render_rime_dictionary(dict_id, &entries)
        };
        (entries, skipped, rendered)
    };

    if entries.is_empty() {
        return Err(RimeError::InvalidDictionaryPath(
            "未解析到有效词条".to_string(),
        ));
    }

    let reference = dictionary_reference_from_name(dict_id);
    Ok((
        dict_name,
        reference,
        entries,
        skipped_entries,
        rendered_contents,
    ))
}

pub(crate) fn url_encode_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

pub(crate) fn sogou_dictionary_url(id: &str, title: &str) -> String {
    format!(
        "https://pinyin.sogou.com/d/dict/download_cell.php?id={}&name={}&f=detail",
        url_encode_component(id),
        url_encode_component(title)
    )
}

pub(crate) fn online_dictionary_categories() -> Vec<OnlineDictionaryCategory> {
    vec![
        OnlineDictionaryCategory {
            id: "167".to_string(),
            title: "城市地名".to_string(),
            description: "城市、道路、机构、景区和方言词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "1".to_string(),
            title: "自然科学".to_string(),
            description: "天文、地理、生物、数学、物理等领域词。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "76".to_string(),
            title: "社会科学".to_string(),
            description: "法律、经济、管理、教育、心理等常见术语。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "96".to_string(),
            title: "工程应用".to_string(),
            description: "计算机、机械、建筑、电子、电力等专业词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "127".to_string(),
            title: "农林渔畜".to_string(),
            description: "农业、林业、渔业、畜牧业词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "132".to_string(),
            title: "医学".to_string(),
            description: "中医、西药、疾病、医疗器械和护理词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "436".to_string(),
            title: "艺术".to_string(),
            description: "绘画、音乐、摄影、戏剧、书法等艺术词。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "154".to_string(),
            title: "运动休闲".to_string(),
            description: "球类、棋牌、武术、垂钓、奥运等词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "389".to_string(),
            title: "生活".to_string(),
            description: "理财、饮食、旅游、办公、手机、美容等词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "367".to_string(),
            title: "娱乐".to_string(),
            description: "动漫、明星、汽车、影视、模型等词库。".to_string(),
        },
    ]
}

pub(crate) fn online_dictionary_catalog() -> Vec<OnlineDictionary> {
    vec![
        OnlineDictionary {
            id: "sogou_trending".to_string(),
            title: "网络流行新词".to_string(),
            category: "通用".to_string(),
            description: "搜狗搜索自动生成的流行新词，每周更新。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_trending.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/4".to_string(),
        },
        OnlineDictionary {
            id: "sogou_computer_all".to_string(),
            title: "计算机词汇大全".to_string(),
            category: "技术".to_string(),
            description: "计算机、软件、网络和信息技术常用词。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_computer_all.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/15117".to_string(),
        },
        OnlineDictionary {
            id: "sogou_developer".to_string(),
            title: "开发大神专用词库".to_string(),
            category: "技术".to_string(),
            description: "程序开发、配置、资源、工具链等开发者常用词。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_developer.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/75228".to_string(),
        },
        OnlineDictionary {
            id: "sogou_programming_all".to_string(),
            title: "计算机编程开发词库大全".to_string(),
            category: "技术".to_string(),
            description: "覆盖 Python、C、JavaScript、Web 全栈和常用算法词汇。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_programming_all.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/133021".to_string(),
        },
        OnlineDictionary {
            id: "sogou_security_3k".to_string(),
            title: "网络安全词库3k".to_string(),
            category: "技术".to_string(),
            description: "网络安全、渗透测试、数据安全、运维等常用词。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_security_3k.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/169975".to_string(),
        },
        OnlineDictionary {
            id: "sogou_car_models".to_string(),
            title: "汽车车型大全".to_string(),
            category: "生活".to_string(),
            description: "汽车品牌、厂商和车型名称。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_car_models.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/22422".to_string(),
        },
        OnlineDictionary {
            id: "sogou_high_school_poems".to_string(),
            title: "高中常考古诗词".to_string(),
            category: "文史".to_string(),
            description: "高中阶段常见古诗词句。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_high_school_poems.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/77212".to_string(),
        },
        OnlineDictionary {
            id: "sogou_middle_school_poems".to_string(),
            title: "中学常见古诗词".to_string(),
            category: "文史".to_string(),
            description: "中学阶段常见古诗词句。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_middle_school_poems.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/74594".to_string(),
        },
    ]
}

pub(crate) fn html_entity_decode(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub(crate) fn strip_html_tags(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    html_entity_decode(output.trim())
}

pub(crate) fn find_between<'a>(contents: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_index = contents.find(start)? + start.len();
    let rest = &contents[start_index..];
    let end_index = rest.find(end)?;
    Some(&rest[..end_index])
}

pub(crate) fn extract_detail_title(block: &str) -> Option<(String, String)> {
    let link = find_between(block, "<div class=\"detail_title\"><a href='", "</a>")?;
    let (href, title_part) = link.split_once("'>")?;
    let id = href.rsplit('/').next()?.trim();
    if id.is_empty() {
        return None;
    }
    Some((id.to_string(), strip_html_tags(title_part)))
}

pub(crate) fn extract_show_contents(block: &str) -> Vec<String> {
    let mut contents = Vec::new();
    let mut rest = block;
    while let Some(start) = rest.find("<div class=\"show_content\">") {
        rest = &rest[start + "<div class=\"show_content\">".len()..];
        let Some(end) = rest.find("</div>") else {
            break;
        };
        contents.push(strip_html_tags(&rest[..end]));
        rest = &rest[end + "</div>".len()..];
    }
    contents
}

pub(crate) fn parse_sogou_category_page(category_id: &str, html: &str) -> Vec<OnlineDictionary> {
    let category_title = online_dictionary_categories()
        .into_iter()
        .find(|category| category.id == category_id)
        .map(|category| category.title)
        .unwrap_or_else(|| "搜狗分类".to_string());
    let mut result = Vec::new();

    for block in html.split("<div class=\"dict_detail_block").skip(1) {
        if let Some((id, title)) = extract_detail_title(block) {
            let show_contents = extract_show_contents(block);
            let sample = show_contents.first().cloned().unwrap_or_default();
            let downloads = show_contents.get(1).cloned().unwrap_or_default();
            let updated = show_contents.get(2).cloned().unwrap_or_default();
            let mut description = sample;
            if !downloads.is_empty() || !updated.is_empty() {
                description = format!(
                    "{}{}{}",
                    description,
                    if downloads.is_empty() {
                        "".to_string()
                    } else {
                        format!(" 下载 {downloads}")
                    },
                    if updated.is_empty() {
                        "".to_string()
                    } else {
                        format!(" 更新 {updated}")
                    }
                )
                .trim()
                .to_string();
            }
            result.push(OnlineDictionary {
                id: format!("sogou_{id}"),
                title,
                category: category_title.clone(),
                description,
                source: "搜狗细胞词库".to_string(),
                source_name: format!("sogou_{id}.scel"),
                detail_url: format!("https://pinyin.sogou.com/dict/detail/index/{id}"),
            });
        }
    }

    result
}

pub(crate) fn sogou_detail_id(detail_url: &str) -> Option<&str> {
    detail_url
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
}

pub(crate) fn online_dictionary_by_id(id: &str) -> Option<OnlineDictionary> {
    online_dictionary_catalog()
        .into_iter()
        .find(|entry| entry.id == id)
}

pub(crate) fn dictionary_source_name_from_url(url: &str, fallback: &str) -> String {
    let without_query = url.split(['?', '#']).next().unwrap_or(url);
    let name = without_query
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    if name.contains('.') {
        name.to_string()
    } else {
        fallback.to_string()
    }
}

pub(crate) fn validate_dictionary_download_url(url: &str) -> Result<(), RimeError> {
    let lower = url.to_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        Ok(())
    } else {
        Err(RimeError::InvalidDictionaryPath(
            "只支持 http:// 或 https:// 在线词库地址".to_string(),
        ))
    }
}

pub(crate) fn download_dictionary_bytes(
    url: &str,
    referer: Option<&str>,
) -> Result<Vec<u8>, RimeError> {
    validate_dictionary_download_url(url)?;
    let mut request = http_agent()
        .get(url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "*/*");
    if let Some(referer) = referer {
        request = request.set("Referer", referer);
    }

    let response = request
        .call()
        .map_err(|err| RimeError::NetworkError(format!("下载在线词库失败: {err}")))?;
    let mut reader = response
        .into_reader()
        .take((MAX_DICTIONARY_DOWNLOAD_BYTES + 1) as u64);
    let mut data = Vec::new();
    reader
        .read_to_end(&mut data)
        .map_err(|err| RimeError::NetworkError(format!("读取在线词库失败: {err}")))?;
    if data.is_empty() {
        return Err(RimeError::DownloadError("在线词库下载结果为空".to_string()));
    }
    if data.len() > MAX_DICTIONARY_DOWNLOAD_BYTES {
        return Err(RimeError::DownloadError(
            "在线词库超过 64MB，已取消导入".to_string(),
        ));
    }
    Ok(data)
}

pub(crate) fn download_url_to_file_with_progress<F>(
    url: &str,
    destination: &Path,
    max_bytes: usize,
    empty_message: &str,
    too_large_message: &str,
    mut progress: F,
) -> Result<(), RimeError>
where
    F: FnMut(u64, Option<u64>),
{
    validate_dictionary_download_url(url)?;
    let response = http_agent()
        .get(url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "*/*")
        .call()
        .map_err(|err| RimeError::DownloadError(format!("下载失败: {err}")))?;
    let total_bytes = response
        .header("Content-Length")
        .and_then(|value| value.parse::<u64>().ok());
    if total_bytes.is_some_and(|value| value > max_bytes as u64) {
        return Err(RimeError::DownloadError(too_large_message.to_string()));
    }

    let mut reader = response.into_reader();
    let mut file = fs::File::create(destination)
        .map_err(|err| RimeError::FileOperationError(format!("创建下载文件失败: {err}")))?;
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded = 0u64;
    let mut last_emit = Instant::now();
    progress(downloaded, total_bytes);

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| RimeError::DownloadError(format!("读取下载内容失败: {err}")))?;
        if read == 0 {
            break;
        }
        downloaded += read as u64;
        if downloaded > max_bytes as u64 {
            let _ = fs::remove_file(destination);
            return Err(RimeError::DownloadError(too_large_message.to_string()));
        }
        file.write_all(&buffer[..read])
            .map_err(|err| RimeError::FileOperationError(format!("保存下载文件失败: {err}")))?;
        if last_emit.elapsed().as_millis() >= 200 || total_bytes == Some(downloaded) {
            progress(downloaded, total_bytes);
            last_emit = Instant::now();
        }
    }

    if downloaded == 0 {
        let _ = fs::remove_file(destination);
        return Err(RimeError::DownloadError(empty_message.to_string()));
    }
    file.flush()
        .map_err(|err| RimeError::FileOperationError(format!("保存下载文件失败: {err}")))?;
    progress(downloaded, total_bytes);
    Ok(())
}

pub(crate) fn resolve_sogou_detail_download(
    url: &str,
    source_name: Option<String>,
) -> Result<Option<(String, String, Option<String>)>, RimeError> {
    let marker = "pinyin.sogou.com/dict/detail/index/";
    let Some(marker_index) = url.find(marker) else {
        return Ok(None);
    };
    let id = url[marker_index + marker.len()..]
        .split(['?', '#', '/'])
        .next()
        .unwrap_or_default()
        .trim();
    if id.is_empty() {
        return Err(RimeError::InvalidDictionaryPath(
            "搜狗词库详情页地址缺少词库 ID".to_string(),
        ));
    }

    let response = http_agent()
        .get(url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "text/html,*/*")
        .call()
        .map_err(|err| RimeError::NetworkError(format!("读取搜狗词库详情页失败: {err}")))?;
    let html = response
        .into_string()
        .map_err(|err| RimeError::NetworkError(format!("解析搜狗词库详情页失败: {err}")))?;
    let title = find_between(&html, "<div class=\"dict_detail_title\">", "</div>")
        .map(strip_html_tags)
        .or_else(|| {
            find_between(&html, "<title>", "</title>")
                .map(strip_html_tags)
                .map(|title| title.replace("_搜狗输入法词库", ""))
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("搜狗词库 {id}"));
    let source_name = source_name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("sogou_{id}.scel"));
    Ok(Some((
        sogou_dictionary_url(id, &title),
        source_name,
        Some(url.to_string()),
    )))
}

pub(crate) fn download_dictionary_import_source(
    url: String,
    source_name: Option<String>,
) -> Result<(String, Vec<u8>), RimeError> {
    if let Some((download_url, source_name, referer)) =
        resolve_sogou_detail_download(&url, source_name.clone())?
    {
        let data = download_dictionary_bytes(&download_url, referer.as_deref())?;
        return Ok((source_name, data));
    }

    let source_name = source_name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| dictionary_source_name_from_url(&url, "online_dictionary.scel"));
    let data = download_dictionary_bytes(&url, None)?;
    Ok((source_name, data))
}

pub(crate) fn download_online_dictionary(entry: &OnlineDictionary) -> Result<Vec<u8>, RimeError> {
    let detail_id = sogou_detail_id(&entry.detail_url)
        .ok_or_else(|| RimeError::InvalidDictionaryPath("在线词库详情地址无效".to_string()))?;
    let url = sogou_dictionary_url(detail_id, &entry.title);
    download_dictionary_bytes(&url, Some(&entry.detail_url))
}

pub(crate) fn list_online_dictionaries_sync() -> Result<Vec<OnlineDictionary>, RimeError> {
    Ok(online_dictionary_catalog())
}

pub(crate) fn list_online_dictionary_categories_sync(
) -> Result<Vec<OnlineDictionaryCategory>, RimeError> {
    Ok(online_dictionary_categories())
}

pub(crate) fn list_online_dictionaries_by_category_sync(
    category_id: String,
) -> Result<Vec<OnlineDictionary>, RimeError> {
    let category = online_dictionary_categories()
        .into_iter()
        .find(|category| category.id == category_id)
        .ok_or_else(|| RimeError::NetworkError("未找到在线词库分类".to_string()))?;
    let url = format!(
        "https://pinyin.sogou.com/dict/cate/index/{}/download",
        category.id
    );
    let response = http_agent()
        .get(&url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "text/html,*/*")
        .call()
        .map_err(|err| RimeError::NetworkError(format!("读取在线词库分类失败: {err}")))?;
    let html = response
        .into_string()
        .map_err(|err| RimeError::NetworkError(format!("解析在线词库分类失败: {err}")))?;
    let dictionaries = parse_sogou_category_page(&category.id, &html);
    if dictionaries.is_empty() {
        Err(RimeError::NetworkError(
            "这个分类没有解析到可导入词库".to_string(),
        ))
    } else {
        Ok(dictionaries)
    }
}

pub(crate) fn preview_online_dictionary_import_sync(
    id: String,
) -> Result<DictionaryImportPreview, RimeError> {
    let entry = online_dictionary_by_id(&id)
        .ok_or_else(|| RimeError::NetworkError("未找到在线词库".to_string()))?;
    let data = download_online_dictionary(&entry)?;
    preview_dictionary_import_sync(entry.source_name, data)
}

pub(crate) fn import_online_dictionary_sync(
    id: String,
) -> Result<DictionaryImportResult, RimeError> {
    let entry = online_dictionary_by_id(&id)
        .ok_or_else(|| RimeError::NetworkError("未找到在线词库".to_string()))?;
    let data = download_online_dictionary(&entry)?;
    import_dictionary_sync(entry.source_name, data)
}

pub(crate) fn preview_dictionary_url_import_sync(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportPreview, RimeError> {
    let (source_name, data) = download_dictionary_import_source(url, source_name)?;
    preview_dictionary_import_sync(source_name, data)
}

pub(crate) fn import_dictionary_url_sync(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportResult, RimeError> {
    let (source_name, data) = download_dictionary_import_source(url, source_name)?;
    import_dictionary_sync(source_name, data)
}
