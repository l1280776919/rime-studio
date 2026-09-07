use crate::backend::*;
use crate::*;

pub(crate) fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub(crate) fn decode_utf16_le(data: &[u8]) -> String {
    let units = data
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                Some(u16::from_le_bytes([chunk[0], chunk[1]]))
            } else {
                None
            }
        })
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
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    let index = u16::from_le_bytes([chunk[0], chunk[1]]);
                    pinyin_table.get(&index).cloned()
                } else {
                    None
                }
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
    for chunk in index_bytes.chunks(2) {
        if chunk.len() < 2 {
            continue;
        }
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

        // Split by Tab, or fallback to space/comma if no Tab is found
        let parts: Vec<&str> = if trimmed.contains('\t') {
            trimmed.split('\t').map(str::trim).collect()
        } else if trimmed.contains(',') {
            trimmed.split(',').map(str::trim).collect()
        } else {
            trimmed.split_whitespace().collect()
        };

        if parts.is_empty() {
            skipped += 1;
            continue;
        }

        let first = parts[0];
        let second = parts.get(1).copied().unwrap_or("");
        let third = parts.get(2).copied().unwrap_or("");

        // Detect if format is (pinyin, word) e.g., Sogou TXT / QQ pinyin export: 'pinyin' 'word'
        let first_is_ascii = first
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '\'' || c == '_' || c == '-');
        let second_is_hanzi = second
            .chars()
            .any(|c| ('\u{4e00}'..='\u{9fa5}').contains(&c));

        let (text, code, weight_str) = if first_is_ascii && second_is_hanzi && !first.is_empty() {
            // Format: (pinyin, word, weight) -> normalize to (word, pinyin, weight)
            let pinyin = first.replace('\'', " ");
            (second.to_string(), pinyin.trim().to_string(), third)
        } else {
            // Standard format: (word, pinyin, weight)
            (first.to_string(), second.to_string(), third)
        };

        if text.is_empty() {
            skipped += 1;
            continue;
        }

        let weight = weight_str.parse::<i32>().unwrap_or(1);
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
