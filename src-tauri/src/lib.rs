use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize)]
struct FileStatus {
    name: String,
    path: String,
    exists: bool,
    size: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DictHealth {
    entries: usize,
    duplicate_exact_lines: usize,
    long_low_weight_entries: usize,
}

#[derive(Debug, Serialize)]
struct RimeEnvironment {
    user_dir: String,
    build_dir: String,
    deployer_path: Option<String>,
    plum_dir: String,
    git_available: bool,
    bash_available: bool,
    git_path: Option<String>,
    bash_path: Option<String>,
    active_schema: Option<String>,
    page_size: Option<u32>,
    theme_name: Option<String>,
    font_point: Option<u32>,
    label_font_point: Option<u32>,
    custom_files: Vec<FileStatus>,
    sogou_health: Option<DictHealth>,
}

#[derive(Debug, Serialize)]
struct DeployResult {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct InstallResult {
    success: bool,
    recipe: String,
    backup_dir: Option<String>,
    log: String,
}

#[derive(Debug, Serialize)]
struct BackupEntry {
    name: String,
    path: String,
    modified: Option<u64>,
    files: usize,
}

#[derive(Debug, Serialize)]
struct RestoreResult {
    restored_files: usize,
    safety_backup_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PhraseEntry {
    text: String,
    code: String,
    weight: i32,
}

#[derive(Debug, Serialize)]
struct DictInfo {
    name: String,
    path: String,
    entry_count: usize,
    size_bytes: u64,
    modified: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppearanceConfig {
    theme_name: String,
    font_point: u32,
    label_font_point: u32,
    page_size: u32,
    horizontal: bool,
    inline_preedit: bool,
    candidate_format: String,
    corner_radius: u32,
    border_height: u32,
    border_width: u32,
    line_spacing: u32,
    spacing: u32,
    back_color: String,
    border_color: String,
    text_color: String,
    candidate_text_color: String,
    comment_text_color: String,
    hilited_text_color: String,
    hilited_back_color: String,
    hilited_candidate_text_color: String,
    hilited_candidate_back_color: String,
}

fn rime_user_dir() -> Result<PathBuf, String> {
    let appdata = env::var("APPDATA").map_err(|_| "APPDATA 环境变量不可用".to_string())?;
    Ok(PathBuf::from(appdata).join("Rime"))
}

fn app_data_dir() -> Result<PathBuf, String> {
    let local_appdata =
        env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA 环境变量不可用".to_string())?;
    Ok(PathBuf::from(local_appdata).join("RimeStudio"))
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn file_status(user_dir: &Path, name: &str) -> FileStatus {
    let path = user_dir.join(name);
    let metadata = fs::metadata(&path).ok();

    FileStatus {
        name: name.to_string(),
        path: path.display().to_string(),
        exists: metadata.is_some(),
        size: metadata.map(|meta| meta.len()),
    }
}

fn parse_schema(default_custom: &str) -> Option<String> {
    default_custom
        .lines()
        .find_map(|line| line.split("{schema:").nth(1))
        .and_then(|rest| rest.split('}').next())
        .map(|schema| schema.trim().to_string())
}

fn parse_u32_after_key(contents: &str, key: &str) -> Option<u32> {
    contents.lines().find_map(|line| {
        if !line.contains(key) {
            return None;
        }

        line.split(':')
            .nth(1)
            .and_then(|value| value.split('#').next())
            .and_then(|value| value.trim().parse::<u32>().ok())
    })
}

fn parse_quoted_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        if !line.contains(key) {
            return None;
        }

        line.split(':')
            .nth(1)
            .map(str::trim)
            .map(|value| value.trim_matches('"').to_string())
    })
}

fn parse_bool_after_key(contents: &str, key: &str) -> Option<bool> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim().trim_matches('"');
        if !trimmed.starts_with(key) {
            return None;
        }

        trimmed
            .split(':')
            .nth(1)
            .and_then(|value| value.split('#').next())
            .map(str::trim)
            .and_then(|value| match value {
                "true" | "True" | "yes" => Some(true),
                "false" | "False" | "no" => Some(false),
                _ => None,
            })
    })
}

fn parse_string_after_key(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim().trim_matches('"');
        if !trimmed.starts_with(key) {
            return None;
        }

        trimmed
            .split(':')
            .nth(1)
            .map(str::trim)
            .map(|value| value.split('#').next().unwrap_or(value).trim())
            .map(|value| value.trim_matches('"').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn normalize_color(value: Option<String>, fallback: &str) -> String {
    value
        .map(|value| {
            value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn locate_deployer() -> Option<PathBuf> {
    let start_menu_shortcut = PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\小狼毫输入法\【小狼毫】重新部署.lnk",
    );

    let known_paths = [
        PathBuf::from(r"D:\xlh\weasel-0.17.4\WeaselDeployer.exe"),
        PathBuf::from(r"C:\Program Files\Rime\weasel-0.17.4\WeaselDeployer.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Rime\weasel-0.17.4\WeaselDeployer.exe"),
    ];

    known_paths
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| {
            if start_menu_shortcut.exists() {
                Some(start_menu_shortcut)
            } else {
                None
            }
        })
}

fn command_success(path: &Path, arg: &str) -> bool {
    Command::new(path)
        .arg(arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_path_success(command: &str, arg: &str) -> bool {
    Command::new(command)
        .arg(arg)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn locate_from_where(command: &str) -> Vec<PathBuf> {
    Command::new("where")
        .arg(command)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

fn git_roots_from_path(path: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = path.parent() {
        if parent.file_name().and_then(OsStr::to_str) == Some("cmd")
            || parent.file_name().and_then(OsStr::to_str) == Some("bin")
        {
            if let Some(root) = parent.parent() {
                roots.push(root.to_path_buf());
            }
        }
    }
    roots
}

fn locate_git() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from(r"C:\Program Files\Git\cmd\git.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Git\cmd\git.exe"),
        PathBuf::from(r"D:\codesoft\Git\cmd\git.exe"),
    ];
    candidates.extend(locate_from_where("git.exe"));
    candidates.extend(locate_from_where("git"));

    candidates
        .into_iter()
        .find(|path| path.exists() && command_success(path, "--version"))
        .or_else(|| {
            if command_path_success("git", "--version") {
                Some(PathBuf::from("git"))
            } else {
                None
            }
        })
}

fn locate_git_bash() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from(r"C:\Program Files\Git\bin\bash.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Git\bin\bash.exe"),
        PathBuf::from(r"D:\codesoft\Git\bin\bash.exe"),
    ];

    for git_path in locate_git().into_iter() {
        for root in git_roots_from_path(&git_path) {
            candidates.push(root.join("bin").join("bash.exe"));
            candidates.push(root.join("usr").join("bin").join("bash.exe"));
        }
    }

    candidates
        .into_iter()
        .find(|path| path.exists() && command_success(path, "--version"))
}

fn read_appearance_config(user_dir: &Path) -> AppearanceConfig {
    let weasel_custom = read_to_string(&user_dir.join("weasel.custom.yaml"));
    let theme_name = parse_string_after_key(&weasel_custom, "style/color_scheme")
        .or_else(|| parse_quoted_value(&weasel_custom, "name:"))
        .unwrap_or_else(|| "rime_studio_blue".to_string());
    let scheme_key = format!("preset_color_schemes/{theme_name}/");

    AppearanceConfig {
        theme_name,
        font_point: parse_u32_after_key(&weasel_custom, "style/font_point").unwrap_or(12),
        label_font_point: parse_u32_after_key(&weasel_custom, "style/label_font_point")
            .unwrap_or(11),
        page_size: parse_u32_after_key(&weasel_custom, "style/page_size").unwrap_or(7),
        horizontal: parse_bool_after_key(&weasel_custom, "style/horizontal").unwrap_or(true),
        inline_preedit: parse_bool_after_key(&weasel_custom, "style/inline_preedit")
            .unwrap_or(true),
        candidate_format: parse_string_after_key(&weasel_custom, "style/candidate_format")
            .unwrap_or_else(|| "%c. %@".to_string()),
        corner_radius: parse_u32_after_key(&weasel_custom, "style/corner_radius").unwrap_or(8),
        border_height: parse_u32_after_key(&weasel_custom, "style/border_height").unwrap_or(4),
        border_width: parse_u32_after_key(&weasel_custom, "style/border_width").unwrap_or(4),
        line_spacing: parse_u32_after_key(&weasel_custom, "style/line_spacing").unwrap_or(6),
        spacing: parse_u32_after_key(&weasel_custom, "style/spacing").unwrap_or(8),
        back_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}back_color")),
            "0xFFF8F0",
        ),
        border_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}border_color")),
            "0xE8CFAF",
        ),
        text_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}text_color")),
            "0x4A2F18",
        ),
        candidate_text_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}candidate_text_color")),
            "0x4A2F18",
        ),
        comment_text_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}comment_text_color")),
            "0x8A735E",
        ),
        hilited_text_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}hilited_text_color")),
            "0xFFFFFF",
        ),
        hilited_back_color: normalize_color(
            parse_string_after_key(&weasel_custom, &format!("{scheme_key}hilited_back_color")),
            "0xD37D2F",
        ),
        hilited_candidate_text_color: normalize_color(
            parse_string_after_key(
                &weasel_custom,
                &format!("{scheme_key}hilited_candidate_text_color"),
            ),
            "0xFFFFFF",
        ),
        hilited_candidate_back_color: normalize_color(
            parse_string_after_key(
                &weasel_custom,
                &format!("{scheme_key}hilited_candidate_back_color"),
            ),
            "0xD37D2F",
        ),
    }
}

fn upsert_patch_value(contents: &str, key: &str, value: &str) -> String {
    let quoted_key = format!("\"{key}\"");
    let mut found = false;
    let mut lines: Vec<String> = contents
        .lines()
        .map(|line| {
            let trimmed = line.trim_start().trim_matches('"');
            if trimmed.starts_with(key) || line.trim_start().starts_with(&quoted_key) {
                found = true;
                let indent_len = line.len() - line.trim_start().len();
                format!("{}{}: {}", " ".repeat(indent_len), key, value)
            } else {
                line.to_string()
            }
        })
        .collect();

    if !found {
        if !lines.iter().any(|line| line.trim() == "patch:") {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.push("patch:".to_string());
        }
        lines.push(format!("  {key}: {value}"));
    }

    let mut next = lines.join("\n");
    next.push('\n');
    next
}

fn write_appearance_config(user_dir: &Path, config: &AppearanceConfig) -> Result<(), String> {
    fs::create_dir_all(user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let path = user_dir.join("weasel.custom.yaml");
    let contents = fs::read_to_string(&path).unwrap_or_else(|_| "patch:\n".to_string());
    let contents = upsert_patch_value(
        &contents,
        "style/color_scheme",
        &format!("\"{}\"", config.theme_name),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/font_point",
        &config.font_point.to_string(),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/label_font_point",
        &config.label_font_point.to_string(),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/page_size",
        &config.page_size.to_string(),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/horizontal",
        if config.horizontal { "true" } else { "false" },
    );
    let contents = upsert_patch_value(
        &contents,
        "style/inline_preedit",
        if config.inline_preedit {
            "true"
        } else {
            "false"
        },
    );
    let contents = upsert_patch_value(
        &contents,
        "style/candidate_format",
        &format!("\"{}\"", config.candidate_format),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/corner_radius",
        &config.corner_radius.to_string(),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/border_height",
        &config.border_height.to_string(),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/border_width",
        &config.border_width.to_string(),
    );
    let contents = upsert_patch_value(
        &contents,
        "style/line_spacing",
        &config.line_spacing.to_string(),
    );
    let contents = upsert_patch_value(&contents, "style/spacing", &config.spacing.to_string());
    let scheme_key = format!("preset_color_schemes/{}/", config.theme_name);
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}name"),
        &format!("\"{}\"", config.theme_name),
    );
    let contents = upsert_patch_value(&contents, &format!("{scheme_key}author"), "\"Rime Studio\"");
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}back_color"),
        &config.back_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}border_color"),
        &config.border_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}text_color"),
        &config.text_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}candidate_text_color"),
        &config.candidate_text_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}comment_text_color"),
        &config.comment_text_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}hilited_text_color"),
        &config.hilited_text_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}hilited_back_color"),
        &config.hilited_back_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}hilited_candidate_text_color"),
        &config.hilited_candidate_text_color,
    );
    let contents = upsert_patch_value(
        &contents,
        &format!("{scheme_key}hilited_candidate_back_color"),
        &config.hilited_candidate_back_color,
    );

    fs::write(&path, contents).map_err(|err| format!("写入外观配置文件失败: {err}"))
}

fn analyze_sogou(path: &Path) -> Option<DictHealth> {
    let contents = fs::read_to_string(path).ok()?;
    let mut entries = 0usize;
    let mut duplicate_exact_lines = 0usize;
    let mut long_low_weight_entries = 0usize;
    let mut seen = std::collections::HashSet::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed == "---"
            || trimmed == "..."
            || trimmed.starts_with("name:")
            || trimmed.starts_with("version:")
            || trimmed.starts_with("sort:")
        {
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
    })
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "unknown-time".to_string())
}

fn copy_if_exists(source: &Path, target: &Path) -> io::Result<()> {
    if source.exists() {
        fs::copy(source, target)?;
    }
    Ok(())
}

fn backup_user_config(user_dir: &Path) -> Result<PathBuf, String> {
    let backup_root = app_data_dir()?;
    let backup_dir = backup_root.join(format!("backup-rime-studio-{}", timestamp()));
    fs::create_dir_all(&backup_dir)
        .map_err(|err| format!("创建备份目录失败: {err}"))?;

    for entry in fs::read_dir(user_dir).map_err(|err| format!("读取 Rime 目录失败: {err}"))? {
        let entry = entry.map_err(|err| format!("检查 Rime 文件失败: {err}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        let should_backup = name.ends_with(".custom.yaml")
            || name.ends_with(".dict.yaml")
            || name == "custom_phrase.txt"
            || name == "default.yaml"
            || name == "weasel.yaml";

        if should_backup {
            copy_if_exists(&path, &backup_dir.join(name))
                .map_err(|err| format!("备份 {name} 失败: {err}"))?;
        }
    }

    Ok(backup_dir)
}

fn list_backup_dirs(_user_dir: &Path) -> Result<Vec<BackupEntry>, String> {
    let backup_root = app_data_dir()?;
    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&backup_root).map_err(|err| format!("读取备份目录失败: {err}"))? {
        let entry = entry.map_err(|err| format!("检查 Rime 文件失败: {err}"))?;
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
                    .filter(|item| item.path().is_file())
                    .count()
            })
            .unwrap_or(0);

        backups.push(BackupEntry {
            name: name.to_string(),
            path: path.display().to_string(),
            modified,
            files,
        });
    }

    backups.sort_by(|left, right| right.modified.cmp(&left.modified));
    Ok(backups)
}

fn validated_backup_dir(_user_dir: &Path, backup_name: &str) -> Result<PathBuf, String> {
    if !backup_name.starts_with("backup-rime-studio-")
        || backup_name.contains('/')
        || backup_name.contains('\\')
        || backup_name.contains("..")
    {
        return Err("无效的备份名称".to_string());
    }

    let backup_root = app_data_dir()?;
    let backup_dir = backup_root.join(backup_name);
    if !backup_dir.is_dir() {
        return Err(format!("备份不存在: {backup_name}"));
    }

    Ok(backup_dir)
}

fn restore_backup_dir(user_dir: &Path, backup_dir: &Path) -> Result<RestoreResult, String> {
    let safety_backup_dir = backup_user_config(user_dir)?;
    let mut restored_files = 0usize;

    for entry in fs::read_dir(backup_dir).map_err(|err| format!("读取备份失败: {err}"))? {
        let entry = entry.map_err(|err| format!("检查备份文件失败: {err}"))?;
        let source = entry.path();
        if !source.is_file() {
            continue;
        }

        let Some(name) = source.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        fs::copy(&source, user_dir.join(name))
            .map_err(|err| format!("恢复 {name} 失败: {err}"))?;
        restored_files += 1;
    }

    Ok(RestoreResult {
        restored_files,
        safety_backup_dir: safety_backup_dir.display().to_string(),
    })
}

fn get_custom_phrases_sync() -> Result<Vec<PhraseEntry>, String> {
    let user_dir = rime_user_dir()?;
    let path = user_dir.join("custom_phrase.txt");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("读取自定义短语文件失败: {err}"))?;

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
    backup_user_config(&user_dir)?;

    let path = user_dir.join("custom_phrase.txt");

    // Preserve comment lines and the Rime header
    let existing_header: String = if path.exists() {
        fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .take_while(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('#') || trimmed.is_empty() || trimmed == "---" || trimmed == "..."
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
    sorted.sort_by(|a, b| b.weight.cmp(&a.weight));

    for phrase in &sorted {
        contents.push_str(&format!(
            "{}\t{}\t{}\n",
            phrase.text, phrase.code, phrase.weight
        ));
    }

    fs::write(&path, contents)
        .map_err(|err| format!("写入自定义短语文件失败: {err}"))
}

fn list_dictionaries_sync() -> Result<Vec<DictInfo>, String> {
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(Vec::new());
    }

    let mut dicts = Vec::new();
    let entries = fs::read_dir(&user_dir)
        .map_err(|err| format!("读取 Rime 目录失败: {err}"))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("检查文件失败: {err}"))?;
        let path = entry.path();
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

        dicts.push(DictInfo {
            name: name.to_string(),
            path: path.display().to_string(),
            entry_count,
            size_bytes,
            modified,
        });
    }

    dicts.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(dicts)
}

fn get_dict_health_sync(dict_name: String) -> Result<DictHealth, String> {
    let user_dir = rime_user_dir()?;
    let valid_name = dict_name
        .replace('/', "")
        .replace('\\', "")
        .replace("..", "");
    let path = user_dir.join(&valid_name);

    if !path.exists() {
        return Err(format!("词库不存在: {valid_name}"));
    }

    analyze_sogou(&path).ok_or_else(|| "词库分析失败".to_string())
}

fn open_in_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    Command::new("explorer")
        .arg(path)
        .spawn()
        .map_err(|err| format!("打开资源管理器失败: {err}"))?;
    Ok(())
}

fn run_command(mut command: Command) -> Result<(bool, String), String> {
    let output = command
        .output()
        .map_err(|err| format!("运行命令失败: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = format!("{stdout}{stderr}");

    Ok((output.status.success(), log))
}

fn ensure_plum(plum_dir: &Path) -> Result<String, String> {
    let git = locate_git()
        .ok_or_else(|| "安装 rime-ice 需要 Git，但未找到".to_string())?;

    let mut log = String::new();
    if plum_dir.join(".git").exists() {
        let mut command = Command::new(&git);
        command.arg("-C").arg(plum_dir).arg("pull").arg("--ff-only");
        let (success, command_log) = run_command(command)?;
        log.push_str(&command_log);
        if !success {
            return Err(format!("更新 plum 失败:\n{log}"));
        }
    } else {
        if let Some(parent) = plum_dir.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("创建应用数据目录失败: {err}"))?;
        }

        let mut command = Command::new(&git);
        command
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/rime/plum.git")
            .arg(plum_dir);
        let (success, command_log) = run_command(command)?;
        log.push_str(&command_log);
        if !success {
            return Err(format!("克隆 plum 失败:\n{log}"));
        }
    }

    Ok(log)
}

fn deploy_rime_internal() -> Result<DeployResult, String> {
    let deployer =
        locate_deployer().ok_or_else(|| "未找到 WeaselDeployer.exe".to_string())?;
    let deployer_path = if deployer.extension().and_then(|ext| ext.to_str()) == Some("lnk") {
        return Err(
            "找到开始菜单快捷方式，但尚未支持快捷方式直接执行"
                .to_string(),
        );
    } else {
        deployer
    };

    let output = Command::new(&deployer_path)
        .arg("/deploy")
        .current_dir(
            deployer_path
                .parent()
                .ok_or_else(|| "部署器路径异常".to_string())?,
        )
        .output()
        .map_err(|err| format!("运行部署器失败: {err}"))?;

    Ok(DeployResult {
        success: output.status.success(),
        message: if output.status.success() {
            "Rime 部署完成".to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        },
    })
}

fn scan_rime_environment_sync() -> Result<RimeEnvironment, String> {
    let user_dir = rime_user_dir()?;
    let build_dir = user_dir.join("build");
    let plum_dir = app_data_dir()?.join("plum");
    let default_custom = read_to_string(&user_dir.join("default.custom.yaml"));
    let appearance = read_appearance_config(&user_dir);
    let git_path = locate_git();
    let bash_path = locate_git_bash();

    Ok(RimeEnvironment {
        user_dir: user_dir.display().to_string(),
        build_dir: build_dir.display().to_string(),
        deployer_path: locate_deployer().map(|path| path.display().to_string()),
        plum_dir: plum_dir.display().to_string(),
        git_available: git_path.is_some(),
        bash_available: bash_path.is_some(),
        git_path: git_path.map(|path| path.display().to_string()),
        bash_path: bash_path.map(|path| path.display().to_string()),
        active_schema: parse_schema(&default_custom),
        page_size: parse_u32_after_key(&default_custom, "menu/page_size"),
        theme_name: Some(appearance.theme_name),
        font_point: Some(appearance.font_point),
        label_font_point: Some(appearance.label_font_point),
        custom_files: [
            "default.custom.yaml",
            "rime_ice.custom.yaml",
            "weasel.custom.yaml",
            "custom_phrase.txt",
            "rime_ice_ext.dict.yaml",
            "sogou_ext.dict.yaml",
        ]
        .into_iter()
        .map(|name| file_status(&user_dir, name))
        .collect(),
        sogou_health: analyze_sogou(&user_dir.join("sogou_ext.dict.yaml")),
    })
}

fn deploy_rime_sync() -> Result<DeployResult, String> {
    deploy_rime_internal()
}

fn install_rime_ice_sync(recipe: Option<String>) -> Result<InstallResult, String> {
    let bash = locate_git_bash();
    if bash.is_none() {
        return Err(
            "运行 rime-install 需要 Git Bash，但未找到".to_string(),
        );
    }
    let bash = bash.unwrap();

    let recipe = recipe.unwrap_or_else(|| "iDvel/rime-ice:others/recipes/full".to_string());
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let backup_dir = backup_user_config(&user_dir)?;
    let plum_dir = app_data_dir()?.join("plum");

    let mut log = String::new();
    log.push_str("正在准备 plum...\n");
    log.push_str(&ensure_plum(&plum_dir)?);
    log.push_str("\n正在安装方案: ");
    log.push_str(&recipe);
    log.push('\n');

    let mut command = Command::new(&bash);
    command
        .arg("rime-install")
        .arg(&recipe)
        .current_dir(&plum_dir)
        .env("rime_dir", &user_dir);
    let (install_success, install_log) = run_command(command)?;
    log.push_str(&install_log);

    if !install_success {
        return Ok(InstallResult {
            success: false,
            recipe,
            backup_dir: Some(backup_dir.display().to_string()),
            log,
        });
    }

    log.push_str("\n正在部署小狼毫...\n");
    match deploy_rime_internal() {
        Ok(result) => {
            log.push_str(&result.message);
            Ok(InstallResult {
                success: result.success,
                recipe,
                backup_dir: Some(backup_dir.display().to_string()),
                log,
            })
        }
        Err(err) => {
            log.push_str(&err);
            Ok(InstallResult {
                success: false,
                recipe,
                backup_dir: Some(backup_dir.display().to_string()),
                log,
            })
        }
    }
}

fn get_appearance_config_sync() -> Result<AppearanceConfig, String> {
    let user_dir = rime_user_dir()?;
    Ok(read_appearance_config(&user_dir))
}

fn save_appearance_config_sync(config: AppearanceConfig) -> Result<AppearanceConfig, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    backup_user_config(&user_dir)?;
    write_appearance_config(&user_dir, &config)?;
    Ok(read_appearance_config(&user_dir))
}

fn list_backups_sync() -> Result<Vec<BackupEntry>, String> {
    let user_dir = rime_user_dir()?;
    list_backup_dirs(&user_dir)
}

fn create_backup_sync() -> Result<BackupEntry, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let backup_dir = backup_user_config(&user_dir)?;
    let backup_name = backup_dir
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("backup-rime-studio")
        .to_string();

    list_backup_dirs(&user_dir)?
        .into_iter()
        .find(|backup| backup.name == backup_name)
        .ok_or_else(|| "备份已创建但无法列出".to_string())
}

fn open_rime_user_dir_sync() -> Result<(), String> {
    open_in_explorer(&rime_user_dir()?)
}

fn open_plum_dir_sync() -> Result<(), String> {
    open_in_explorer(&app_data_dir()?.join("plum"))
}

fn open_backup_dir_sync(backup_name: String) -> Result<(), String> {
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    open_in_explorer(&backup_dir)
}

fn restore_backup_sync(backup_name: String) -> Result<RestoreResult, String> {
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    restore_backup_dir(&user_dir, &backup_dir)
}

fn scan_dictionaries_sync_wrapper() -> Result<Vec<DictInfo>, String> {
    list_dictionaries_sync()
}

fn get_dict_health_sync_wrapper(dict_name: String) -> Result<DictHealth, String> {
    get_dict_health_sync(dict_name)
}

#[derive(Debug, Serialize)]
struct RimeDownloadResult {
    success: bool,
    installer_path: Option<String>,
    message: String,
}

fn download_rime_installer_sync() -> Result<RimeDownloadResult, String> {
    // Fetch latest release info from GitHub API
    let api_url = "https://api.github.com/repos/rime/weasel/releases/latest";
    let response = ureq::get(api_url)
        .set("User-Agent", "RimeStudio/0.1")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|err| format!("获取 Rime 发布信息失败: {err}"))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|err| format!("解析发布信息失败: {err}"))?;

    // Find the .exe installer asset
    let assets = json["assets"].as_array().ok_or("未找到发布资源")?;
    let installer = assets
        .iter()
        .find_map(|asset| {
            let name = asset["name"].as_str().unwrap_or("");
            if name.ends_with(".exe") && name.contains("install") {
                Some((name.to_string(), asset["browser_download_url"].as_str()?.to_string()))
            } else if name.ends_with(".exe") {
                Some((name.to_string(), asset["browser_download_url"].as_str()?.to_string()))
            } else {
                None
            }
        })
        .ok_or("未找到合适的安装包")?;

    let download_url = installer.1;
    let filename = installer.0;

    // Download to app data dir
    let dest_dir = app_data_dir()?;
    fs::create_dir_all(&dest_dir)
        .map_err(|err| format!("创建下载目录失败: {err}"))?;
    let dest_path = dest_dir.join(&filename);

    // Download with progress
    let response = ureq::get(&download_url)
        .set("User-Agent", "RimeStudio/0.1")
        .call()
        .map_err(|err| format!("下载失败: {err}"))?;

    let mut reader = response.into_reader();
    let mut file = fs::File::create(&dest_path)
        .map_err(|err| format!("创建文件失败: {err}"))?;
    std::io::copy(&mut reader, &mut file)
        .map_err(|err| format!("保存文件失败: {err}"))?;

    Ok(RimeDownloadResult {
        success: true,
        installer_path: Some(dest_path.display().to_string()),
        message: format!("已下载 {filename}"),
    })
}

fn launch_installer_sync(path: String) -> Result<(), String> {
    let installer_path = PathBuf::from(&path);
    if !installer_path.exists() {
        return Err("安装包文件不存在".to_string());
    }

    Command::new(&installer_path)
        .spawn()
        .map_err(|err| format!("启动安装程序失败: {err}"))?;

    Ok(())
}

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("后台任务失败: {err}"))?
}

#[tauri::command]
async fn scan_rime_environment() -> Result<RimeEnvironment, String> {
    run_blocking(scan_rime_environment_sync).await
}

#[tauri::command]
async fn deploy_rime() -> Result<DeployResult, String> {
    run_blocking(deploy_rime_sync).await
}

#[tauri::command]
async fn install_rime_ice(recipe: Option<String>) -> Result<InstallResult, String> {
    run_blocking(move || install_rime_ice_sync(recipe)).await
}

#[tauri::command]
async fn get_appearance_config() -> Result<AppearanceConfig, String> {
    run_blocking(get_appearance_config_sync).await
}

#[tauri::command]
async fn save_appearance_config(config: AppearanceConfig) -> Result<AppearanceConfig, String> {
    run_blocking(move || save_appearance_config_sync(config)).await
}

#[tauri::command]
async fn list_backups() -> Result<Vec<BackupEntry>, String> {
    run_blocking(list_backups_sync).await
}

#[tauri::command]
async fn create_backup() -> Result<BackupEntry, String> {
    run_blocking(create_backup_sync).await
}

#[tauri::command]
async fn open_rime_user_dir() -> Result<(), String> {
    run_blocking(open_rime_user_dir_sync).await
}

#[tauri::command]
async fn open_plum_dir() -> Result<(), String> {
    run_blocking(open_plum_dir_sync).await
}

#[tauri::command]
async fn open_backup_dir(backup_name: String) -> Result<(), String> {
    run_blocking(move || open_backup_dir_sync(backup_name)).await
}

#[tauri::command]
async fn restore_backup(backup_name: String) -> Result<RestoreResult, String> {
    run_blocking(move || restore_backup_sync(backup_name)).await
}

#[tauri::command]
async fn get_custom_phrases() -> Result<Vec<PhraseEntry>, String> {
    run_blocking(get_custom_phrases_sync).await
}

#[tauri::command]
async fn save_custom_phrases(phrases: Vec<PhraseEntry>) -> Result<(), String> {
    run_blocking(move || save_custom_phrases_sync(phrases)).await
}

#[tauri::command]
async fn list_dictionaries() -> Result<Vec<DictInfo>, String> {
    run_blocking(scan_dictionaries_sync_wrapper).await
}

#[tauri::command]
async fn get_dict_health(dict_name: String) -> Result<DictHealth, String> {
    run_blocking(move || get_dict_health_sync_wrapper(dict_name)).await
}

#[tauri::command]
async fn download_rime_installer() -> Result<RimeDownloadResult, String> {
    run_blocking(download_rime_installer_sync).await
}

#[tauri::command]
async fn launch_rime_installer(path: String) -> Result<(), String> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_rime_environment,
            deploy_rime,
            install_rime_ice,
            get_appearance_config,
            save_appearance_config,
            list_backups,
            create_backup,
            open_rime_user_dir,
            open_plum_dir,
            open_backup_dir,
            restore_backup,
            get_custom_phrases,
            save_custom_phrases,
            list_dictionaries,
            get_dict_health,
            download_rime_installer,
            launch_rime_installer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
