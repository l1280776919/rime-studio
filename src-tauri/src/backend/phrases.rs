use crate::backend::*;
use crate::*;
use std::fs;

fn is_phrase_metadata(line: &str) -> bool {
    let trimmed = line.trim().trim_start_matches('\u{feff}');
    trimmed.is_empty() || trimmed.starts_with('#') || matches!(trimmed, "---" | "...")
}

fn parse_phrases(contents: &str) -> Result<Vec<PhraseEntry>, RimeError> {
    let mut phrases = Vec::new();
    for (index, line) in contents.trim_start_matches('\u{feff}').lines().enumerate() {
        if is_phrase_metadata(line) {
            continue;
        }
        // Trimming the line would lose phrase spaces and shift empty TSV columns.
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() > 3 || parts[0].trim().is_empty() {
            return Err(RimeError::FileOperationError(format!(
                "自定义短语第 {} 行格式无效，已中止读取",
                index + 1
            )));
        }
        let weight = match parts.get(2).map(|part| part.trim()) {
            None | Some("") => 0,
            Some(value) => value.parse::<i32>().map_err(|_| {
                RimeError::FileOperationError(format!("自定义短语第 {} 行权重无效", index + 1))
            })?,
        };
        phrases.push(PhraseEntry {
            text: parts[0].to_string(),
            code: parts.get(1).copied().unwrap_or_default().to_string(),
            weight,
        });
    }
    Ok(phrases)
}

pub(crate) fn get_custom_phrases_sync() -> Result<Vec<PhraseEntry>, RimeError> {
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(Vec::new());
    }
    let path = resolve_user_relative_path(&user_dir, "custom_phrase.txt", false)?;
    parse_phrases(&read_optional_config(&path)?)
}

pub(crate) fn save_custom_phrases_sync(phrases: Vec<PhraseEntry>) -> Result<(), RimeError> {
    for phrase in &phrases {
        if is_phrase_metadata(&phrase.text)
            || phrase.text.contains(['\t', '\r', '\n', '\0'])
            || phrase.code.contains(['\t', '\r', '\n', '\0'])
        {
            return Err(RimeError::FileOperationError(
                "短语内容或编码格式无效：不得为空、使用注释/头部标记，或包含制表符、换行及空字符"
                    .into(),
            ));
        }
    }
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let path = resolve_user_relative_path(&user_dir, "custom_phrase.txt", false)?;
    let previous = read_optional_config(&path)?;
    // Preserve all annotations, including comments between entries.
    let header = previous
        .trim_start_matches('\u{feff}')
        .lines()
        .filter(|line| is_phrase_metadata(line))
        .collect::<Vec<_>>()
        .join("\n");
    let mut contents = if header.is_empty() {
        String::from("# Rime 自定义短语\n# 格式: 短语\\t编码\\t权重\n")
    } else {
        format!("{header}\n")
    };
    for phrase in &phrases {
        contents.push_str(&format!(
            "{}\t{}\t{}\n",
            phrase.text, phrase.code, phrase.weight
        ));
    }
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(&path, &contents, "写入自定义短语文件失败")
}
