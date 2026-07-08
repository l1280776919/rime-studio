fn get_custom_phrases_sync() -> Result<Vec<PhraseEntry>, String> {
    let user_dir = rime_user_dir()?;
    let path = user_dir.join("custom_phrase.txt");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents =
        fs::read_to_string(&path).map_err(|err| format!("读取自定义短语文件失败: {err}"))?;

    let mut phrases = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.is_empty() || parts[0].is_empty() {
            continue;
        }

        let text = parts[0].to_string();
        let code = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
        let weight = parts
            .get(2)
            .and_then(|s| s.trim().parse::<i32>().ok())
            .unwrap_or(0);

        phrases.push(PhraseEntry { text, code, weight });
    }

    Ok(phrases)
}

fn save_custom_phrases_sync(phrases: Vec<PhraseEntry>) -> Result<(), String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    let path = user_dir.join("custom_phrase.txt");

    // Preserve comment lines and the Rime header
    let existing_header: String = if path.exists() {
        fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .take_while(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('#')
                    || trimmed.is_empty()
                    || trimmed == "---"
                    || trimmed == "..."
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    let mut contents = if existing_header.is_empty() {
        String::from("# Rime 自定义短语\n# 格式: 短语\\t编码\\t权重\n")
    } else {
        format!("{existing_header}\n")
    };

    let mut sorted = phrases;
    sorted.sort_by_key(|b| std::cmp::Reverse(b.weight));

    for phrase in &sorted {
        contents.push_str(&format!(
            "{}\t{}\t{}\n",
            phrase.text, phrase.code, phrase.weight
        ));
    }

    write_text_file(&path, &contents, "写入自定义短语文件失败")
}

