use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

type DictionaryEntry = (String, String, i32);

const SOGOU_BIN_PINYIN: &[&str] = &[
    "a", "ai", "an", "ang", "ao", "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi",
    "bian", "biao", "bie", "bin", "bing", "bo", "bu", "ca", "cai", "can", "cang", "cao", "ce",
    "cen", "ceng", "cha", "chai", "chan", "chang", "chao", "che", "chen", "cheng", "chi", "chong",
    "chou", "chu", "chua", "chuai", "chuan", "chuang", "chui", "chun", "chuo", "ci", "cong", "cou",
    "cu", "cuan", "cui", "cun", "cuo", "da", "dai", "dan", "dang", "dao", "de", "dei", "den",
    "deng", "di", "dia", "dian", "diao", "die", "ding", "diu", "dong", "dou", "du", "duan", "dui",
    "dun", "duo", "e", "ei", "en", "eng", "er", "fa", "fan", "fang", "fei", "fen", "feng", "fiao",
    "fo", "fou", "fu", "ga", "gai", "gan", "gang", "gao", "ge", "gei", "gen", "geng", "gong",
    "gou", "gu", "gua", "guai", "guan", "guang", "gui", "gun", "guo", "ha", "hai", "han", "hang",
    "hao", "he", "hei", "hen", "heng", "hong", "hou", "hu", "hua", "huai", "huan", "huang", "hui",
    "hun", "huo", "ji", "jia", "jian", "jiang", "jiao", "jie", "jin", "jing", "jiong", "jiu", "ju",
    "juan", "jue", "jun", "ka", "kai", "kan", "kang", "kao", "ke", "kei", "ken", "keng", "kong",
    "kou", "ku", "kua", "kuai", "kuan", "kuang", "kui", "kun", "kuo", "la", "lai", "lan", "lang",
    "lao", "le", "lei", "leng", "li", "lia", "lian", "liang", "liao", "lie", "lin", "ling", "liu",
    "lo", "long", "lou", "lu", "luan", "lve", "lun", "luo", "lv", "ma", "mai", "man", "mang",
    "mao", "me", "mei", "men", "meng", "mi", "mian", "miao", "mie", "min", "ming", "miu", "mo",
    "mou", "mu", "na", "nai", "nan", "nang", "nao", "ne", "nei", "nen", "neng", "ni", "nian",
    "niang", "niao", "nie", "nin", "ning", "niu", "nong", "nou", "nu", "nuan", "nve", "nun", "nuo",
    "nv", "o", "ou", "pa", "pai", "pan", "pang", "pao", "pei", "pen", "peng", "pi", "pian", "piao",
    "pie", "pin", "ping", "po", "pou", "pu", "qi", "qia", "qian", "qiang", "qiao", "qie", "qin",
    "qing", "qiong", "qiu", "qu", "quan", "que", "qun", "ran", "rang", "rao", "re", "ren", "reng",
    "ri", "rong", "rou", "ru", "rua", "ruan", "rui", "run", "ruo", "sa", "sai", "san", "sang",
    "sao", "se", "sen", "seng", "sha", "shai", "shan", "shang", "shao", "she", "shei", "shen",
    "sheng", "shi", "shou", "shu", "shua", "shuai", "shuan", "shuang", "shui", "shun", "shuo",
    "si", "song", "sou", "su", "suan", "sui", "sun", "suo", "ta", "tai", "tan", "tang", "tao",
    "te", "tei", "teng", "ti", "tian", "tiao", "tie", "ting", "tong", "tou", "tu", "tuan", "tui",
    "tun", "tuo", "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu", "xi", "xia",
    "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan", "xue", "xun",
    "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yo", "yong", "you", "yu", "yuan",
    "yue", "yun", "za", "zai", "zan", "zang", "zao", "ze", "zei", "zen", "zeng", "zha", "zhai",
    "zhan", "zhang", "zhao", "zhe", "zhei", "zhen", "zheng", "zhi", "zhong", "zhou", "zhu", "zhua",
    "zhuai", "zhuan", "zhuang", "zhui", "zhun", "zhuo", "zi", "zong", "zou", "zu", "zuan", "zui",
    "zun", "zuo",
];

#[derive(Debug, Serialize)]
struct FileStatus {
    name: String,
    path: String,
    exists: bool,
    size: Option<u64>,
    modified: Option<u64>,
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
    kind: String,
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

#[derive(Debug, Serialize)]
struct DictionaryImportResult {
    name: String,
    reference: String,
    path: String,
    imported_entries: usize,
    skipped_entries: usize,
}

#[derive(Debug, Serialize)]
struct DictionaryExportResult {
    name: String,
    contents: String,
}

#[derive(Debug, Serialize)]
struct DictionaryCleanResult {
    name: String,
    path: String,
    removed_duplicate_lines: usize,
    entries_after: usize,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppearanceConfig {
    theme_name: String,
    font_point: u32,
    label_font_point: u32,
    page_size: u32,
    switch_key: String,
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

#[derive(Debug, Deserialize, Serialize)]
struct QuickSettingsConfig {
    schema_id: String,
    page_size: u32,
    switch_key: String,
    paging_keys: String,
    navigation_keys: String,
    horizontal: bool,
    inline_preedit: bool,
}

#[derive(Debug, Serialize)]
struct ConfigHealthCheck {
    name: String,
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct ConfigHealthReport {
    summary: String,
    checks: Vec<ConfigHealthCheck>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RimeIceSettings {
    emoji: bool,
    traditionalization: bool,
    ascii_punct: bool,
    full_shape: bool,
    search_single_char: bool,
    traditional_preset: String,
}

#[derive(Debug, Serialize)]
struct DictionaryReference {
    reference: String,
    path: Option<String>,
    exists: bool,
    entry_count: Option<usize>,
    size_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DictionaryConfig {
    schema_id: Option<String>,
    schema_name: Option<String>,
    main_dictionary: Option<String>,
    main_dictionary_path: Option<String>,
    enabled: Vec<DictionaryReference>,
    available: Vec<DictInfo>,
    missing: Vec<DictionaryReference>,
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

fn yaml_mapping_get<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

fn yaml_path_get<'a>(value: &'a Value, key_path: &str) -> Option<&'a Value> {
    let mut current = value;
    for key in key_path.split('/') {
        let Value::Mapping(mapping) = current else {
            return None;
        };
        current = yaml_mapping_get(mapping, key)?;
    }
    Some(current)
}

fn yaml_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn yaml_lookup(contents: &str, key: &str) -> Option<Value> {
    let key = key.trim_matches('"').trim_end_matches(':');
    let document = serde_yaml::from_str::<Value>(contents).ok()?;

    yaml_path_get(&document, key)
        .or_else(|| {
            let patch = yaml_path_get(&document, "patch")?;
            match patch {
                Value::Mapping(mapping) => yaml_mapping_get(mapping, key),
                _ => None,
            }
        })
        .or_else(|| {
            let patch = yaml_path_get(&document, "patch")?;
            yaml_path_get(patch, key)
        })
        .cloned()
}

fn suppress_console_window(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

fn file_status(user_dir: &Path, name: &str) -> FileStatus {
    let path = user_dir.join(name);
    let metadata = fs::metadata(&path).ok();

    FileStatus {
        name: name.to_string(),
        path: path.display().to_string(),
        exists: metadata.is_some(),
        size: metadata.as_ref().map(|meta| meta.len()),
        modified: metadata
            .and_then(|meta| meta.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs()),
    }
}

fn parse_schema(default_custom: &str) -> Option<String> {
    parse_schema_list(default_custom).into_iter().next()
}

fn parse_schema_list(default_custom: &str) -> Vec<String> {
    if let Some(Value::Sequence(schema_list)) = yaml_lookup(default_custom, "schema_list") {
        let schemas = schema_list
            .iter()
            .filter_map(|item| match item {
                Value::Mapping(mapping) => yaml_mapping_get(mapping, "schema"),
                _ => None,
            })
            .filter_map(yaml_value_to_string)
            .filter(|schema| !schema.is_empty())
            .collect::<Vec<_>>();
        if !schemas.is_empty() {
            return schemas;
        }
    }

    default_custom
        .lines()
        .filter_map(|line| {
            line.split("{schema:")
                .nth(1)
                .and_then(|rest| rest.split('}').next())
                .map(|schema| schema.trim().to_string())
                .filter(|schema| !schema.is_empty())
        })
        .collect()
}

fn parse_u32_after_key(contents: &str, key: &str) -> Option<u32> {
    if let Some(value) = yaml_lookup(contents, key)
        .and_then(|value| yaml_value_to_string(&value))
        .and_then(|value| value.parse::<u32>().ok())
    {
        return Some(value);
    }

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
    if let Some(value) = yaml_lookup(contents, key).and_then(|value| yaml_value_to_string(&value)) {
        return Some(value);
    }

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
    if let Some(value) = yaml_lookup(contents, key) {
        match value {
            Value::Bool(value) => return Some(value),
            Value::String(value) => match value.as_str() {
                "true" | "True" | "yes" => return Some(true),
                "false" | "False" | "no" => return Some(false),
                _ => {}
            },
            _ => {}
        }
    }

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
    if let Some(value) = yaml_lookup(contents, key)
        .and_then(|value| yaml_value_to_string(&value))
        .filter(|value| !value.is_empty())
    {
        return Some(value);
    }

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

fn weasel_deployers_under(root: &Path) -> Vec<PathBuf> {
    fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(|name| name.starts_with("weasel-"))
                    .unwrap_or(false)
        })
        .map(|path| path.join("WeaselDeployer.exe"))
        .filter(|path| path.exists())
        .collect()
}

fn resolve_windows_shortcut(path: &Path) -> Option<PathBuf> {
    if path.extension().and_then(OsStr::to_str) != Some("lnk") {
        return Some(path.to_path_buf());
    }

    let script = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}'); $s.TargetPath",
        path.display().to_string().replace('\'', "''")
    );
    let mut command = Command::new("powershell");
    command.arg("-NoProfile").arg("-Command").arg(script);
    suppress_console_window(&mut command)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if target.is_empty() {
                None
            } else {
                Some(PathBuf::from(target))
            }
        })
        .filter(|target| target.exists())
}

fn locate_deployer() -> Option<PathBuf> {
    let start_menu_shortcut = PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\小狼毫输入法\【小狼毫】重新部署.lnk",
    );

    let mut candidates = vec![
        PathBuf::from(r"D:\xlh\weasel-0.17.4\WeaselDeployer.exe"),
        PathBuf::from(r"D:\soft\rime\weasel-0.17.4\WeaselDeployer.exe"),
        PathBuf::from(r"C:\Program Files\Rime\weasel-0.17.4\WeaselDeployer.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Rime\weasel-0.17.4\WeaselDeployer.exe"),
    ];
    candidates.extend(weasel_deployers_under(&PathBuf::from(r"D:\soft\rime")));
    candidates.extend(weasel_deployers_under(&PathBuf::from(
        r"C:\Program Files\Rime",
    )));
    candidates.extend(weasel_deployers_under(&PathBuf::from(
        r"C:\Program Files (x86)\Rime",
    )));
    if start_menu_shortcut.exists() {
        candidates.push(start_menu_shortcut);
    }

    candidates
        .into_iter()
        .filter(|path| path.exists())
        .find_map(|path| resolve_windows_shortcut(&path))
}

fn command_success(path: &Path, arg: &str) -> bool {
    let mut command = Command::new(path);
    command.arg(arg);
    suppress_console_window(&mut command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_path_success(command: &str, arg: &str) -> bool {
    let mut command = Command::new(command);
    command.arg(arg);
    suppress_console_window(&mut command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn locate_from_where(command: &str) -> Vec<PathBuf> {
    let mut where_command = Command::new("where");
    where_command.arg(command);
    suppress_console_window(&mut where_command)
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
        font_point: parse_u32_after_key(&weasel_custom, "style/font_point").unwrap_or(11),
        label_font_point: parse_u32_after_key(&weasel_custom, "style/label_font_point")
            .unwrap_or(10),
        page_size: {
            let default_custom = read_to_string(&user_dir.join("default.custom.yaml"));
            parse_u32_after_key(&weasel_custom, "style/page_size")
                .or_else(|| parse_u32_after_key(&default_custom, "menu/page_size"))
                .unwrap_or(7)
        },
        switch_key: {
            let dc = read_to_string(&user_dir.join("default.custom.yaml"));
            let val = parse_string_after_key(&dc, "ascii_composer/switch_key/Shift_L");
            val.unwrap_or_else(|| "shift".to_string())
        },
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

fn render_weasel_custom(config: &AppearanceConfig) -> String {
    let scheme_key = format!("preset_color_schemes/{}/", config.theme_name);
    let mut lines = vec![
        "# Managed by Rime Studio. Previous versions are kept in RimeStudio backups.".to_string(),
        "patch:".to_string(),
        format!("  \"style/color_scheme\": \"{}\"", config.theme_name),
        format!("  \"style/font_point\": {}", config.font_point),
        format!("  \"style/label_font_point\": {}", config.label_font_point),
        format!(
            "  \"style/horizontal\": {}",
            if config.horizontal { "true" } else { "false" }
        ),
        format!(
            "  \"style/inline_preedit\": {}",
            if config.inline_preedit {
                "true"
            } else {
                "false"
            }
        ),
        format!(
            "  \"style/candidate_format\": \"{}\"",
            config.candidate_format
        ),
        format!("  \"style/corner_radius\": {}", config.corner_radius),
        format!("  \"style/border_height\": {}", config.border_height),
        format!("  \"style/border_width\": {}", config.border_width),
        format!("  \"style/line_spacing\": {}", config.line_spacing),
        format!("  \"style/spacing\": {}", config.spacing),
        format!("  \"{scheme_key}name\": \"{}\"", config.theme_name),
        format!("  \"{scheme_key}author\": \"Rime Studio\""),
        format!("  \"{scheme_key}back_color\": {}", config.back_color),
        format!("  \"{scheme_key}border_color\": {}", config.border_color),
        format!("  \"{scheme_key}text_color\": {}", config.text_color),
        format!(
            "  \"{scheme_key}candidate_text_color\": {}",
            config.candidate_text_color
        ),
        format!(
            "  \"{scheme_key}comment_text_color\": {}",
            config.comment_text_color
        ),
        format!(
            "  \"{scheme_key}hilited_text_color\": {}",
            config.hilited_text_color
        ),
        format!(
            "  \"{scheme_key}hilited_back_color\": {}",
            config.hilited_back_color
        ),
        format!(
            "  \"{scheme_key}hilited_candidate_text_color\": {}",
            config.hilited_candidate_text_color
        ),
        format!(
            "  \"{scheme_key}hilited_candidate_back_color\": {}",
            config.hilited_candidate_back_color
        ),
    ];
    lines.push(String::new());
    lines.join("\n")
}

fn write_appearance_config(
    user_dir: &Path,
    config: &AppearanceConfig,
    include_behavior: bool,
) -> Result<(), String> {
    fs::create_dir_all(user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let path = user_dir.join("weasel.custom.yaml");
    let _ = include_behavior;
    write_text_file(&path, &render_weasel_custom(config), "写入外观配置文件失败")
}

fn is_dictionary_entry_line(trimmed: &str) -> bool {
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && trimmed != "---"
        && trimmed != "..."
        && !trimmed.starts_with("name:")
        && !trimmed.starts_with("version:")
        && !trimmed.starts_with("sort:")
        && trimmed.contains('\t')
}

fn analyze_sogou(path: &Path) -> Option<DictHealth> {
    let contents = fs::read_to_string(path).ok()?;
    let mut entries = 0usize;
    let mut duplicate_exact_lines = 0usize;
    let mut long_low_weight_entries = 0usize;
    let mut seen = std::collections::HashSet::new();

    for line in contents.lines() {
        let trimmed = line.trim();
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

#[derive(Clone, Copy)]
enum BackupKind {
    Manual,
    BeforeSave,
    BeforeRestore,
    BeforeInstall,
}

impl BackupKind {
    fn as_str(self) -> &'static str {
        match self {
            BackupKind::Manual => "manual",
            BackupKind::BeforeSave => "before-save",
            BackupKind::BeforeRestore => "before-restore",
            BackupKind::BeforeInstall => "before-install",
        }
    }
}

fn backup_kind_from_name(name: &str) -> String {
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

fn create_unique_backup_dir(backup_root: &Path, kind: BackupKind) -> Result<PathBuf, String> {
    for suffix in 0..100 {
        let base = format!("backup-rime-studio-{}-{}", kind.as_str(), timestamp());
        let name = if suffix == 0 {
            base
        } else {
            format!("{base}-{suffix}")
        };
        let path = backup_root.join(name);
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|err| format!("创建备份目录失败: {err}"))?;
            return Ok(path);
        }
    }

    Err("创建备份目录失败: 无法生成唯一目录名".to_string())
}

fn write_text_file(path: &Path, contents: &str, context: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{context}: 目标路径无效"))?;
    fs::create_dir_all(parent).map_err(|err| format!("{context}: 创建目录失败: {err}"))?;

    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| format!("{context}: 文件名无效"))?;
    let temp_path = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        process::id(),
        timestamp()
    ));

    fs::write(&temp_path, contents).map_err(|err| format!("{context}: {err}"))?;
    if let Err(rename_err) = fs::rename(&temp_path, path) {
        if path.exists() {
            fs::remove_file(path).map_err(|remove_err| {
                let _ = fs::remove_file(&temp_path);
                format!("{context}: 替换旧文件失败: {remove_err}")
            })?;
            fs::rename(&temp_path, path).map_err(|retry_err| {
                let _ = fs::remove_file(&temp_path);
                format!("{context}: {retry_err}")
            })?;
        } else {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("{context}: {rename_err}"));
        }
    }

    Ok(())
}

fn is_managed_config_file(name: &str) -> bool {
    name.ends_with(".custom.yaml")
        || name.ends_with(".dict.yaml")
        || name == "custom_phrase.txt"
        || name == "default.yaml"
        || name == "weasel.yaml"
}

fn backup_user_config(user_dir: &Path, kind: BackupKind) -> Result<PathBuf, String> {
    let backup_root = app_data_dir()?;
    fs::create_dir_all(&backup_root).map_err(|err| format!("创建备份根目录失败: {err}"))?;
    let backup_dir = create_unique_backup_dir(&backup_root, kind)?;

    for entry in fs::read_dir(user_dir).map_err(|err| format!("读取 Rime 目录失败: {err}"))? {
        let entry = entry.map_err(|err| format!("检查 Rime 文件失败: {err}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        if is_managed_config_file(name) {
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
    for entry in fs::read_dir(&backup_root).map_err(|err| format!("读取备份目录失败: {err}"))?
    {
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
            kind: backup_kind_from_name(name),
            modified,
            files,
        });
    }

    backups.sort_by_key(|right| std::cmp::Reverse(right.modified));
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
    let safety_backup_dir = backup_user_config(user_dir, BackupKind::BeforeRestore)?;
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
        if !is_managed_config_file(name) {
            continue;
        }

        fs::copy(&source, user_dir.join(name)).map_err(|err| format!("恢复 {name} 失败: {err}"))?;
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

fn list_dictionaries_sync() -> Result<Vec<DictInfo>, String> {
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(Vec::new());
    }

    let mut dicts = Vec::new();
    let mut pending_dirs = vec![user_dir.clone()];

    while let Some(dir) = pending_dirs.pop() {
        let entries = fs::read_dir(&dir).map_err(|err| format!("读取 Rime 目录失败: {err}"))?;

        for entry in entries {
            let entry = entry.map_err(|err| format!("检查文件失败: {err}"))?;
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

fn validate_dictionary_path(user_dir: &Path, dict_name: &str) -> Result<PathBuf, String> {
    if !dict_name.ends_with(".dict.yaml") {
        return Err("只能操作 .dict.yaml 词库文件".to_string());
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
        return Err("词库路径无效".to_string());
    }

    let path = user_dir.join(relative);
    if !path.exists() || !path.is_file() {
        return Err("词库文件不存在".to_string());
    }

    Ok(path)
}

fn dictionary_reference_from_name(name: &str) -> String {
    name.trim_end_matches(".dict.yaml").replace('\\', "/")
}

fn dictionary_file_name_from_reference(reference: &str) -> String {
    format!("{}.dict.yaml", reference.trim_end_matches(".dict.yaml"))
}

fn parse_import_tables(contents: &str) -> Vec<String> {
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

fn resolve_schema_path(user_dir: &Path, schema_id: &str) -> Option<PathBuf> {
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

fn current_schema_dictionary(user_dir: &Path) -> (Option<String>, Option<String>, Option<String>) {
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

fn dict_info_to_reference(info: &DictInfo) -> String {
    dictionary_reference_from_name(&info.name)
}

fn read_dictionary_config_sync() -> Result<DictionaryConfig, String> {
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

fn render_main_dictionary(dictionary_id: &str, imports: &[String]) -> String {
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

fn save_dictionary_imports_sync(imports: Vec<String>) -> Result<DictionaryConfig, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let (_, _, main_dictionary) = current_schema_dictionary(&user_dir);
    let main_dictionary = main_dictionary.ok_or_else(|| "当前方案未找到主词库".to_string())?;
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

fn add_dictionary_to_current_schema_sync(reference: String) -> Result<DictionaryConfig, String> {
    let config = read_dictionary_config_sync()?;
    let reference = reference
        .trim()
        .trim_end_matches(".dict.yaml")
        .replace('\\', "/");
    if reference.is_empty() {
        return Err("词库引用不能为空".to_string());
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

fn remove_dictionary_from_current_schema_sync(
    reference: String,
) -> Result<DictionaryConfig, String> {
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

fn get_dict_health_sync(dict_name: String) -> Result<DictHealth, String> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;

    analyze_sogou(&path).ok_or_else(|| "词库分析失败".to_string())
}

fn remove_duplicate_dictionary_lines(contents: &str) -> (String, usize) {
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

fn clean_dictionary_duplicates_sync(dict_name: String) -> Result<DictionaryCleanResult, String> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;
    let contents = fs::read_to_string(&path).map_err(|err| format!("读取词库失败: {err}"))?;
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

fn sanitize_dict_id(source_name: &str) -> String {
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

fn sanitize_dict_file_name(source_name: &str) -> String {
    let id = sanitize_dict_id(source_name);
    if id.ends_with(".dict") {
        format!("{id}.yaml")
    } else if id.ends_with("_dict") {
        format!("{}.yaml", id.replace("_dict", ".dict"))
    } else {
        format!("{id}.dict.yaml")
    }
}

fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn decode_utf16_le(data: &[u8]) -> String {
    let units = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
        .trim_matches(char::from(0))
        .trim()
        .to_string()
}

fn parse_scel_entries(data: &[u8]) -> Result<(Vec<DictionaryEntry>, usize), String> {
    const PINYIN_TABLE_OFFSET: usize = 0x1540;
    const WORD_TABLE_OFFSET: usize = 0x2628;

    if data.len() <= WORD_TABLE_OFFSET {
        return Err("搜狗 .scel 文件过小或格式不正确".to_string());
    }

    let mut pinyin_table = std::collections::HashMap::<u16, String>::new();
    let mut offset = PINYIN_TABLE_OFFSET;
    while offset + 4 <= data.len() && offset < WORD_TABLE_OFFSET {
        let Some(index) = read_u16_le(data, offset) else {
            break;
        };
        let Some(byte_len) = read_u16_le(data, offset + 2).map(usize::from) else {
            break;
        };
        offset += 4;
        if byte_len == 0 || offset + byte_len > data.len() || offset + byte_len > WORD_TABLE_OFFSET
        {
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
    offset = WORD_TABLE_OFFSET;

    while offset + 4 <= data.len() {
        let Some(same_pinyin_count) = read_u16_le(data, offset).map(usize::from) else {
            break;
        };
        let Some(pinyin_byte_len) = read_u16_le(data, offset + 2).map(usize::from) else {
            break;
        };
        offset += 4;
        if same_pinyin_count == 0 || pinyin_byte_len == 0 || offset + pinyin_byte_len > data.len() {
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
        Err("未能从搜狗 .scel 文件中解析出词条".to_string())
    } else {
        Ok((entries, skipped))
    }
}

fn is_sogou_bin_word(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= 80
        && value.chars().all(|ch| {
            let code = ch as u32;
            (0x4e00..=0x9fff).contains(&code)
                || (0x3400..=0x4dbf).contains(&code)
                || (0x20..=0x7e).contains(&code)
        })
}

fn sogou_bin_code_from_indexes(index_bytes: &[u8]) -> Option<String> {
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

fn parse_sogou_bin_entries(data: &[u8]) -> Result<(Vec<DictionaryEntry>, usize), String> {
    if !data.starts_with(b"SGPU") {
        return Err("不是支持的搜狗用户词库 .bin 备份文件".to_string());
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
        return Err("未能从搜狗 .bin 备份中解析出可导入词条".to_string());
    }

    let entries = weighted
        .into_iter()
        .map(|((word, code), weight)| (word, code, weight))
        .collect();
    Ok((entries, skipped))
}

fn parse_text_dictionary_entries(contents: &str) -> (Vec<DictionaryEntry>, usize) {
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

fn render_rime_dictionary(dict_id: &str, entries: &[DictionaryEntry]) -> String {
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

fn import_dictionary_sync(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportResult, String> {
    if data.is_empty() {
        return Err("导入文件为空".to_string());
    }
    if data.len() > 64 * 1024 * 1024 {
        return Err("导入文件超过 64MB".to_string());
    }

    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;

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
            "文本词库需要使用 UTF-8 编码；搜狗二进制词库请导入 .scel 文件".to_string()
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
        return Err("未解析到有效词条".to_string());
    }

    let path = user_dir.join(&dict_name);
    write_text_file(&path, &rendered_contents, "写入导入词库失败")?;

    Ok(DictionaryImportResult {
        reference: dictionary_reference_from_name(&dict_name),
        name: dict_name,
        path: path.display().to_string(),
        imported_entries: entries.len(),
        skipped_entries,
    })
}

fn export_dictionary_sync(dict_name: String) -> Result<DictionaryExportResult, String> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;

    let contents = fs::read_to_string(&path).map_err(|err| format!("读取词库失败: {err}"))?;
    Ok(DictionaryExportResult {
        name: path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("dictionary.dict.yaml")
            .to_string(),
        contents,
    })
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

fn reveal_in_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }

    Command::new("explorer")
        .arg("/select,")
        .arg(path)
        .spawn()
        .map_err(|err| format!("打开资源管理器失败: {err}"))?;
    Ok(())
}

fn run_command(mut command: Command) -> Result<(bool, String), String> {
    let output = suppress_console_window(&mut command)
        .output()
        .map_err(|err| format!("运行命令失败: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = format!("{stdout}{stderr}");

    Ok((output.status.success(), log))
}

fn ensure_plum(plum_dir: &Path) -> Result<String, String> {
    let git = locate_git().ok_or_else(|| "安装 rime-ice 需要 Git，但未找到".to_string())?;

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
            fs::create_dir_all(parent).map_err(|err| format!("创建应用数据目录失败: {err}"))?;
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
    let deployer_path = locate_deployer().ok_or_else(|| "未找到 WeaselDeployer.exe".to_string())?;

    let mut command = Command::new(&deployer_path);
    command.arg("/deploy").current_dir(
        deployer_path
            .parent()
            .ok_or_else(|| "部署器路径异常".to_string())?,
    );
    suppress_console_window(&mut command)
        .spawn()
        .map_err(|err| format!("运行部署器失败: {err}"))?;

    Ok(DeployResult {
        success: true,
        message: "已启动小狼毫重新部署，请稍候查看候选窗变化".to_string(),
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
            "rime_ice.schema.yaml",
            "rime_ice.dict.yaml",
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
        return Err("运行 rime-install 需要 Git Bash，但未找到".to_string());
    }
    let bash = bash.unwrap();

    let recipe = recipe.unwrap_or_else(|| "iDvel/rime-ice:others/recipes/full".to_string());
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let backup_dir = backup_user_config(&user_dir, BackupKind::BeforeInstall)?;
    let backup_dir_display = backup_dir.display().to_string();
    let plum_dir = app_data_dir()?.join("plum");

    let mut log = String::new();
    log.push_str("已创建安装前备份: ");
    log.push_str(&backup_dir_display);
    log.push('\n');
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
            backup_dir: Some(backup_dir_display.clone()),
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
                backup_dir: Some(backup_dir_display.clone()),
                log,
            })
        }
        Err(err) => {
            log.push_str(&err);
            Ok(InstallResult {
                success: false,
                recipe,
                backup_dir: Some(backup_dir_display.clone()),
                log,
            })
        }
    }
}

fn get_appearance_config_sync() -> Result<AppearanceConfig, String> {
    let user_dir = rime_user_dir()?;
    Ok(read_appearance_config(&user_dir))
}

fn detect_paging_keys(contents: &str) -> String {
    if contents.contains("accept: Up") && contents.contains("send: Page_Up") {
        return "arrow_keys".to_string();
    }
    if contents.contains("accept: minus") && contents.contains("send: Page_Up") {
        return "minus_equal".to_string();
    }
    "comma_period".to_string()
}

fn detect_navigation_keys(contents: &str) -> String {
    // left_right when Left→Up (synthesize Up for selection) OR Left→Page_Up (extra paging)
    let left_sends_up = contents.contains("accept: Left") && contents.contains("send: Up");
    let left_sends_page = contents.contains("accept: Left") && contents.contains("send: Page_Up");
    if left_sends_up || left_sends_page {
        return "left_right".to_string();
    }
    "up_down".to_string()
}

fn get_quick_settings_sync() -> Result<QuickSettingsConfig, String> {
    let user_dir = rime_user_dir()?;
    let default_custom = read_to_string(&user_dir.join("default.custom.yaml"));
    let appearance = read_appearance_config(&user_dir);
    let switch_left = parse_string_after_key(&default_custom, "ascii_composer/switch_key/Shift_L")
        .unwrap_or_else(|| "commit_code".to_string());
    let switch_key = if switch_left == "commit_code" {
        "shift"
    } else {
        "none"
    };
    let paging_keys = detect_paging_keys(&default_custom);

    let navigation_keys = detect_navigation_keys(&default_custom);

    Ok(QuickSettingsConfig {
        schema_id: parse_schema(&default_custom).unwrap_or_else(|| "rime_ice".to_string()),
        page_size: parse_u32_after_key(&default_custom, "menu/page_size")
            .unwrap_or(appearance.page_size),
        switch_key: switch_key.to_string(),
        paging_keys,
        navigation_keys,
        horizontal: appearance.horizontal,
        inline_preedit: appearance.inline_preedit,
    })
}

fn save_quick_settings_sync(config: QuickSettingsConfig) -> Result<QuickSettingsConfig, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    let default_custom_path = user_dir.join("default.custom.yaml");
    let schema_id = config.schema_id.replace(['/', '\\'], "").replace("..", "");
    let switch_value = if config.switch_key == "shift" {
        "commit_code"
    } else {
        "noop"
    };
    let mut default_contents = vec![
        "# Managed by Rime Studio. Previous versions are kept in RimeStudio backups.".to_string(),
        "patch:".to_string(),
        "  \"schema_list\":".to_string(),
        format!("    - {{schema: {schema_id}}}"),
        format!("  \"menu/page_size\": {}", config.page_size),
        format!("  \"ascii_composer/switch_key/Shift_L\": {switch_value}"),
        format!("  \"ascii_composer/switch_key/Shift_R\": {switch_value}"),
    ];
    // Build key_binder bindings for paging and navigation
    let mut bindings: Vec<String> = Vec::new();
    let arrow_paging = config.paging_keys == "arrow_keys";
    let left_right_nav = config.navigation_keys == "left_right";

    if arrow_paging && left_right_nav {
        // Full arrow swap: Up/Down page, Left/Right synthesize Up/Down for selection
        bindings.push("    - {when: paging, accept: Up, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Down, send: Page_Down}".to_string());
        bindings.push("    - {when: has_menu, accept: Left, send: Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Right, send: Down}".to_string());
    } else if arrow_paging {
        // Up/Down page only
        bindings.push("    - {when: paging, accept: Up, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Down, send: Page_Down}".to_string());
    } else if left_right_nav {
        // Left/Right as additional paging keys (Up/Down still select)
        bindings.push("    - {when: has_menu, accept: Left, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Right, send: Page_Down}".to_string());
    } else if config.paging_keys == "minus_equal" {
        // Minus/equal paging
        bindings.push("    - {when: paging, accept: minus, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: equal, send: Page_Down}".to_string());
    }

    if !bindings.is_empty() {
        default_contents.push("  \"key_binder/bindings\":".to_string());
        default_contents.extend(bindings);
    }
    default_contents.push(String::new());
    write_text_file(
        &default_custom_path,
        &default_contents.join("\n"),
        "写入 default.custom.yaml 失败",
    )?;

    let mut appearance = read_appearance_config(&user_dir);
    appearance.page_size = config.page_size;
    appearance.switch_key = config.switch_key;
    appearance.horizontal = config.horizontal;
    appearance.inline_preedit = config.inline_preedit;
    write_appearance_config(&user_dir, &appearance, true)?;

    get_quick_settings_sync()
}

fn push_check(checks: &mut Vec<ConfigHealthCheck>, name: &str, status: &str, detail: String) {
    checks.push(ConfigHealthCheck {
        name: name.to_string(),
        status: status.to_string(),
        detail,
    });
}

fn patch_preamble_is_clean(contents: &str) -> bool {
    contents
        .lines()
        .take_while(|line| line.trim() != "patch:")
        .all(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
}

fn count_patch_key(contents: &str, key: &str) -> usize {
    contents
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start().trim_matches('"');
            trimmed.starts_with(key)
        })
        .count()
}

fn first_patch_string(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim_start().trim_matches('"');
        if !trimmed.starts_with(key) {
            return None;
        }
        line.split_once(':')
            .map(|(_, value)| value.trim().trim_matches('"').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn first_plain_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with(key) {
            return None;
        }
        trimmed
            .split_once(':')
            .map(|(_, value)| value.trim().trim_matches('"').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn nested_plain_value(contents: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == format!("{section}:") {
            in_section = true;
            continue;
        }
        if in_section && !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            in_section = false;
        }
        if in_section && trimmed.starts_with(key) {
            return trimmed
                .split_once(':')
                .map(|(_, value)| value.trim().trim_matches('"').to_string())
                .filter(|value| !value.is_empty());
        }
    }
    None
}

fn inspect_config_health_sync() -> Result<ConfigHealthReport, String> {
    let user_dir = rime_user_dir()?;
    let default_custom_path = user_dir.join("default.custom.yaml");
    let weasel_custom_path = user_dir.join("weasel.custom.yaml");
    let rime_ice_custom_path = user_dir.join("rime_ice.custom.yaml");
    let build_default_path = user_dir.join("build").join("default.yaml");
    let build_weasel_path = user_dir.join("build").join("weasel.yaml");

    let default_custom = read_to_string(&default_custom_path);
    let weasel_custom = read_to_string(&weasel_custom_path);
    let rime_ice_custom = read_to_string(&rime_ice_custom_path);
    let build_default = read_to_string(&build_default_path);
    let build_weasel = read_to_string(&build_weasel_path);

    let mut checks = Vec::new();

    for (label, path, contents) in [
        ("default.custom.yaml", &default_custom_path, &default_custom),
        ("weasel.custom.yaml", &weasel_custom_path, &weasel_custom),
    ] {
        if !path.exists() {
            push_check(
                &mut checks,
                label,
                "warning",
                "文件不存在，保存设置后会自动创建".to_string(),
            );
            continue;
        }
        if !contents.lines().any(|line| line.trim() == "patch:") {
            push_check(
                &mut checks,
                label,
                "error",
                "缺少顶层 patch:，Rime 不会合并自定义配置".to_string(),
            );
        } else if !patch_preamble_is_clean(contents) {
            push_check(
                &mut checks,
                label,
                "error",
                "patch: 前存在非注释内容，可能导致 YAML 结构无效".to_string(),
            );
        } else {
            push_check(&mut checks, label, "ok", "patch 入口看起来正常".to_string());
        }
    }

    let schema_count = count_patch_key(&default_custom, "schema_list");
    if schema_count > 1 {
        push_check(
            &mut checks,
            "方案列表",
            "error",
            format!("schema_list 出现 {schema_count} 次，建议重新保存快速设置"),
        );
    } else if default_custom.contains("\"schema_list\": [") {
        push_check(
            &mut checks,
            "方案列表",
            "warning",
            "检测到旧的一行 schema_list 写法，建议重新保存快速设置".to_string(),
        );
    } else {
        push_check(
            &mut checks,
            "方案列表",
            "ok",
            "未发现明显结构冲突".to_string(),
        );
    }

    let color_count = count_patch_key(&weasel_custom, "style/color_scheme");
    if color_count > 1 {
        push_check(
            &mut checks,
            "主题配置",
            "error",
            format!("style/color_scheme 出现 {color_count} 次，建议重新保存主题"),
        );
    } else {
        push_check(
            &mut checks,
            "主题配置",
            "ok",
            "主题 patch 未发现重复键".to_string(),
        );
    }

    if rime_ice_custom_path.exists() {
        if !rime_ice_custom.lines().any(|line| line.trim() == "patch:") {
            push_check(
                &mut checks,
                "雾凇组件配置",
                "error",
                "rime_ice.custom.yaml 缺少 patch:，组件开关不会被 Rime 合并".to_string(),
            );
        } else if !patch_preamble_is_clean(&rime_ice_custom) {
            push_check(
                &mut checks,
                "雾凇组件配置",
                "error",
                "rime_ice.custom.yaml 的 patch: 前存在非注释内容，可能导致 YAML 结构无效"
                    .to_string(),
            );
        } else {
            let duplicate_switches = [
                "switches/@1/reset",
                "switches/@2/reset",
                "switches/@3/reset",
                "switches/@4/reset",
                "switches/@5/reset",
            ]
            .into_iter()
            .filter(|key| count_patch_key(&rime_ice_custom, key) > 1)
            .count();
            if duplicate_switches > 0 {
                push_check(
                    &mut checks,
                    "雾凇组件配置",
                    "error",
                    format!("发现 {duplicate_switches} 个重复的雾凇开关，建议重新保存雾凇组件"),
                );
            } else {
                push_check(
                    &mut checks,
                    "雾凇组件配置",
                    "ok",
                    "组件开关 patch 看起来正常".to_string(),
                );
            }
        }

        if let Some(preset) = first_patch_string(&rime_ice_custom, "traditionalize/opencc_config") {
            let valid_presets = ["s2t.json", "s2tw.json", "s2twp.json", "s2hk.json"];
            if valid_presets.contains(&preset.as_str()) {
                push_check(
                    &mut checks,
                    "繁体预设",
                    "ok",
                    format!("OpenCC 预设为 {preset}"),
                );
            } else {
                push_check(
                    &mut checks,
                    "繁体预设",
                    "warning",
                    format!("未识别的 OpenCC 预设 {preset}，保存雾凇组件可恢复为常见预设"),
                );
            }
        }
    } else {
        push_check(
            &mut checks,
            "雾凇组件配置",
            "warning",
            "尚未生成 rime_ice.custom.yaml，保存雾凇组件后会自动创建".to_string(),
        );
    }

    if build_weasel_path.exists() {
        let custom_scheme = first_patch_string(&weasel_custom, "style/color_scheme");
        let built_scheme = first_plain_value(&build_weasel, "color_scheme");
        match (custom_scheme, built_scheme) {
            (Some(expected), Some(actual)) if expected == actual => {
                push_check(
                    &mut checks,
                    "主题合并",
                    "ok",
                    format!("build/weasel.yaml 已使用 {actual}"),
                );
            }
            (Some(expected), Some(actual)) => {
                push_check(
                    &mut checks,
                    "主题合并",
                    "error",
                    format!("custom 要求 {expected}，但 build 仍是 {actual}"),
                );
            }
            _ => push_check(
                &mut checks,
                "主题合并",
                "warning",
                "无法读取 custom 或 build 中的主题值".to_string(),
            ),
        }
    } else {
        push_check(
            &mut checks,
            "主题合并",
            "warning",
            "build/weasel.yaml 不存在，尚未部署".to_string(),
        );
    }

    if build_default_path.exists() {
        let custom_page_size = first_patch_string(&default_custom, "menu/page_size")
            .and_then(|value| value.parse::<u32>().ok());
        let built_page_size = nested_plain_value(&build_default, "menu", "page_size")
            .and_then(|value| value.parse::<u32>().ok());
        match (custom_page_size, built_page_size) {
            (Some(expected), Some(actual)) if expected == actual => {
                push_check(
                    &mut checks,
                    "候选数量合并",
                    "ok",
                    format!("build/default.yaml 已使用 {actual}"),
                );
            }
            (Some(expected), Some(actual)) => {
                push_check(
                    &mut checks,
                    "候选数量合并",
                    "error",
                    format!("custom 要求 {expected}，但 build 仍是 {actual}"),
                );
            }
            _ => push_check(
                &mut checks,
                "候选数量合并",
                "warning",
                "无法读取 custom 或 build 中的候选数量".to_string(),
            ),
        }
    } else {
        push_check(
            &mut checks,
            "候选数量合并",
            "warning",
            "build/default.yaml 不存在，尚未部署".to_string(),
        );
    }

    let has_error = checks.iter().any(|check| check.status == "error");
    let has_warning = checks.iter().any(|check| check.status == "warning");
    let summary = if has_error {
        "发现配置阻断项".to_string()
    } else if has_warning {
        "配置基本可用，但有提醒".to_string()
    } else {
        "配置看起来正常".to_string()
    };

    Ok(ConfigHealthReport { summary, checks })
}

fn repair_config_health_sync() -> Result<ConfigHealthReport, String> {
    let quick = get_quick_settings_sync().unwrap_or(QuickSettingsConfig {
        schema_id: "luna_pinyin_simp".to_string(),
        page_size: 5,
        switch_key: "shift".to_string(),
        paging_keys: "comma_period".to_string(),
        navigation_keys: "up_down".to_string(),
        horizontal: true,
        inline_preedit: true,
    });
    let appearance = get_appearance_config_sync()?;

    save_quick_settings_sync(quick)?;
    save_appearance_config_sync(appearance)?;
    let _ = deploy_rime_internal();

    inspect_config_health_sync()
}

fn parse_patch_bool(contents: &str, key: &str, fallback: bool) -> bool {
    first_patch_string(contents, key)
        .and_then(|value| value.parse::<u32>().ok())
        .map(|value| value != 0)
        .unwrap_or(fallback)
}

fn get_rime_ice_settings_sync() -> Result<RimeIceSettings, String> {
    let user_dir = rime_user_dir()?;
    let custom = read_to_string(&user_dir.join("rime_ice.custom.yaml"));
    Ok(RimeIceSettings {
        ascii_punct: parse_patch_bool(&custom, "switches/@1/reset", false),
        traditionalization: parse_patch_bool(&custom, "switches/@2/reset", false),
        emoji: parse_patch_bool(&custom, "switches/@3/reset", true),
        full_shape: parse_patch_bool(&custom, "switches/@4/reset", false),
        search_single_char: parse_patch_bool(&custom, "switches/@5/reset", false),
        traditional_preset: first_patch_string(&custom, "traditionalize/opencc_config")
            .unwrap_or_else(|| "s2t.json".to_string()),
    })
}

fn render_rime_ice_custom(settings: &RimeIceSettings) -> String {
    let bool_to_reset = |value: bool| if value { 1 } else { 0 };
    [
        "# Managed by Rime Studio. Previous versions are kept in RimeStudio backups.".to_string(),
        "patch:".to_string(),
        format!(
            "  \"switches/@1/reset\": {}",
            bool_to_reset(settings.ascii_punct)
        ),
        format!(
            "  \"switches/@2/reset\": {}",
            bool_to_reset(settings.traditionalization)
        ),
        format!("  \"switches/@3/reset\": {}", bool_to_reset(settings.emoji)),
        format!(
            "  \"switches/@4/reset\": {}",
            bool_to_reset(settings.full_shape)
        ),
        format!(
            "  \"switches/@5/reset\": {}",
            bool_to_reset(settings.search_single_char)
        ),
        format!(
            "  \"traditionalize/opencc_config\": \"{}\"",
            settings.traditional_preset
        ),
        String::new(),
    ]
    .join("\n")
}

fn save_rime_ice_settings_sync(settings: RimeIceSettings) -> Result<RimeIceSettings, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(
        &user_dir.join("rime_ice.custom.yaml"),
        &render_rime_ice_custom(&settings),
        "写入 rime_ice.custom.yaml 失败",
    )?;
    Ok(settings)
}

fn save_appearance_config_sync(config: AppearanceConfig) -> Result<AppearanceConfig, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_appearance_config(&user_dir, &config, false)?;

    Ok(read_appearance_config(&user_dir))
}

fn list_backups_sync() -> Result<Vec<BackupEntry>, String> {
    let user_dir = rime_user_dir()?;
    list_backup_dirs(&user_dir)
}

fn create_backup_sync() -> Result<BackupEntry, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let backup_dir = backup_user_config(&user_dir, BackupKind::Manual)?;
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

fn open_config_file_sync(name: String) -> Result<(), String> {
    let allowed = [
        "default.custom.yaml",
        "weasel.custom.yaml",
        "rime_ice.custom.yaml",
        "custom_phrase.txt",
        "rime_ice.schema.yaml",
        "rime_ice.dict.yaml",
        "rime_ice_ext.dict.yaml",
        "sogou_ext.dict.yaml",
    ];
    if !allowed.contains(&name.as_str()) {
        return Err("不支持打开这个配置文件".to_string());
    }

    let path = rime_user_dir()?.join(name);
    if !path.exists() || !path.is_file() {
        return Err("配置文件不存在".to_string());
    }

    reveal_in_explorer(&path)
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

fn delete_backup_sync(backup_name: String) -> Result<(), String> {
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    fs::remove_dir_all(&backup_dir).map_err(|err| format!("删除备份失败: {err}"))
}

fn delete_dictionary_sync(dict_name: String) -> Result<(), String> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;
    fs::remove_file(&path).map_err(|err| format!("删除词库失败: {err}"))
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
        .filter_map(|asset| {
            let name = asset["name"].as_str().unwrap_or("");
            if name.ends_with(".exe") {
                Some((
                    name.to_string(),
                    asset["browser_download_url"].as_str()?.to_string(),
                ))
            } else {
                None
            }
        })
        .max_by_key(|(name, _)| name.contains("install"))
        .ok_or("未找到合适的安装包")?;

    let download_url = installer.1;
    let filename = installer.0;

    // Download to app data dir
    let dest_dir = app_data_dir()?;
    fs::create_dir_all(&dest_dir).map_err(|err| format!("创建下载目录失败: {err}"))?;
    let dest_path = dest_dir.join(&filename);

    // Download with progress
    let response = ureq::get(&download_url)
        .set("User-Agent", "RimeStudio/0.1")
        .call()
        .map_err(|err| format!("下载失败: {err}"))?;

    let mut reader = response.into_reader();
    let mut file = fs::File::create(&dest_path).map_err(|err| format!("创建文件失败: {err}"))?;
    std::io::copy(&mut reader, &mut file).map_err(|err| format!("保存文件失败: {err}"))?;

    Ok(RimeDownloadResult {
        success: true,
        installer_path: Some(dest_path.display().to_string()),
        message: format!("已下载 {filename}"),
    })
}

fn validate_downloaded_installer_path(path: String) -> Result<PathBuf, String> {
    let installer_path = PathBuf::from(path);
    if !installer_path.exists() || !installer_path.is_file() {
        return Err("安装包文件不存在".to_string());
    }
    if installer_path.extension().and_then(OsStr::to_str) != Some("exe") {
        return Err("只能启动 Rime Studio 下载的 .exe 安装包".to_string());
    }

    let app_dir = app_data_dir()?;
    let canonical_app_dir = app_dir
        .canonicalize()
        .map_err(|err| format!("读取下载目录失败: {err}"))?;
    let canonical_installer = installer_path
        .canonicalize()
        .map_err(|err| format!("读取安装包路径失败: {err}"))?;

    if !canonical_installer.starts_with(canonical_app_dir) {
        return Err("只能启动 Rime Studio 下载目录内的安装包".to_string());
    }

    Ok(canonical_installer)
}

fn launch_installer_sync(path: String) -> Result<(), String> {
    let installer_path = validate_downloaded_installer_path(path)?;

    Command::new(&installer_path)
        .spawn()
        .map_err(|err| format!("启动安装程序失败: {err}"))?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct SchemaInfo {
    id: String,
    name: String,
    description: String,
    path: String,
    is_system: bool,
    is_active: bool,
    is_enabled: bool,
}

fn list_schemas_sync() -> Result<Vec<SchemaInfo>, String> {
    let user_dir = rime_user_dir()?;
    let active_schema = read_to_string(&user_dir.join("default.custom.yaml"));
    let active = parse_schema(&active_schema);
    let enabled = parse_schema_list(&active_schema);
    let mut schemas = Vec::new();

    // Find system schemas from Weasel data directory
    let system_dirs: Vec<PathBuf> = locate_deployer()
        .and_then(|d| d.parent().map(|p| p.join("data")))
        .into_iter()
        .chain(std::iter::once(PathBuf::from(
            r"C:\Program Files\Rime\weasel-0.17.4\data",
        )))
        .chain(std::iter::once(PathBuf::from(
            r"C:\Program Files (x86)\Rime\weasel-0.17.4\data",
        )))
        .collect();

    let mut seen = std::collections::HashSet::new();

    // Scan system data dirs
    for data_dir in &system_dirs {
        if !data_dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                    continue;
                };
                if !name.ends_with(".schema.yaml") {
                    continue;
                }
                let id = name.replace(".schema.yaml", "");
                if seen.contains(&id) {
                    continue;
                }
                seen.insert(id.clone());

                let contents = fs::read_to_string(&path).unwrap_or_default();
                let schema_name =
                    parse_quoted_value(&contents, "schema/name").unwrap_or_else(|| id.clone());
                let description = parse_quoted_value(&contents, "schema/description")
                    .or_else(|| parse_string_after_key(&contents, "description:"))
                    .unwrap_or_default();

                schemas.push(SchemaInfo {
                    is_system: true,
                    is_active: active.as_ref() == Some(&id),
                    is_enabled: enabled.iter().any(|schema_id| schema_id == &id),
                    id,
                    name: schema_name,
                    description,
                    path: path.display().to_string(),
                });
            }
        }
    }

    // Scan user dir for custom schemas
    if user_dir.exists() {
        if let Ok(entries) = fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                    continue;
                };
                if !name.ends_with(".schema.yaml") && !name.ends_with(".custom.yaml") {
                    continue;
                }
                // Skip weasel.custom.yaml and default.custom.yaml
                if name == "weasel.custom.yaml" || name == "default.custom.yaml" {
                    continue;
                }
                let id = name.replace(".custom.yaml", "").replace(".schema.yaml", "");
                if seen.contains(&id) {
                    // Already listed as system; mark as having user override
                    if name.ends_with(".custom.yaml") {
                        if let Some(s) = schemas.iter_mut().find(|s| s.id == id) {
                            s.is_system = false;
                        }
                    }
                    continue;
                }
                seen.insert(id.clone());

                let contents = fs::read_to_string(&path).unwrap_or_default();
                let schema_name = parse_quoted_value(&contents, "schema/name")
                    .or_else(|| parse_string_after_key(&contents, "name:"))
                    .unwrap_or_else(|| id.clone());
                let description = parse_quoted_value(&contents, "schema/description")
                    .or_else(|| parse_string_after_key(&contents, "description:"))
                    .unwrap_or_default();

                schemas.push(SchemaInfo {
                    is_system: false,
                    is_active: active.as_ref() == Some(&id),
                    is_enabled: enabled.iter().any(|schema_id| schema_id == &id),
                    id,
                    name: schema_name,
                    description,
                    path: path.display().to_string(),
                });
            }
        }
    }

    schemas.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then(a.is_system.cmp(&b.is_system))
            .then(a.name.cmp(&b.name))
    });
    Ok(schemas)
}

fn copy_schema_sync(schema_id: String) -> Result<String, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;

    let safe_id = schema_id.replace(['/', '\\'], "").replace("..", "");

    // Find the source schema file
    let system_dirs = [
        locate_deployer().and_then(|d| d.parent().map(|p| p.join("data"))),
        Some(PathBuf::from(r"C:\Program Files\Rime\weasel-0.17.4\data")),
        Some(PathBuf::from(
            r"C:\Program Files (x86)\Rime\weasel-0.17.4\data",
        )),
    ];

    let mut source: Option<PathBuf> = None;
    for dir in system_dirs.iter().flatten() {
        let candidate = dir.join(format!("{safe_id}.schema.yaml"));
        if candidate.exists() {
            source = Some(candidate);
            break;
        }
    }

    // Also check user dir
    let user_candidate = user_dir.join(format!("{safe_id}.schema.yaml"));
    if user_candidate.exists() {
        source = Some(user_candidate);
    }

    let source = source.ok_or("未找到源方案文件".to_string())?;

    // Read source and create a custom copy
    let contents = fs::read_to_string(&source).map_err(|err| format!("读取方案文件失败: {err}"))?;

    // Write as .custom.yaml in user dir
    let dest_name = format!("{safe_id}.custom.yaml");
    let dest = user_dir.join(&dest_name);

    if dest.exists() {
        return Err(format!("{dest_name} 已存在，未自动覆盖"));
    }

    // Add a header comment
    let patched = format!(
        "# {} — 从系统方案复制，由 Rime Studio 管理\n# 在此文件中添加 patch 配置即可自定义方案\n\n{}",
        safe_id, contents
    );

    write_text_file(&dest, &patched, "写入方案文件失败")?;
    Ok(dest.display().to_string())
}

fn sanitize_schema_id(schema_id: &str) -> String {
    schema_id
        .replace(['/', '\\'], "")
        .replace("..", "")
        .trim()
        .to_string()
}

fn sanitize_schema_ids(schema_ids: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    schema_ids
        .into_iter()
        .map(|schema_id| sanitize_schema_id(&schema_id))
        .filter(|schema_id| !schema_id.is_empty())
        .filter(|schema_id| seen.insert(schema_id.clone()))
        .collect()
}

fn render_default_custom_with_schema_list(
    config: &QuickSettingsConfig,
    schema_ids: &[String],
) -> String {
    let switch_value = if config.switch_key == "shift" {
        "commit_code"
    } else {
        "noop"
    };
    let mut default_contents = vec![
        "# Managed by Rime Studio. Previous versions are kept in RimeStudio backups.".to_string(),
        "patch:".to_string(),
        "  \"schema_list\":".to_string(),
    ];

    for schema_id in schema_ids {
        default_contents.push(format!("    - {{schema: {schema_id}}}"));
    }

    default_contents.extend([
        format!("  \"menu/page_size\": {}", config.page_size),
        format!("  \"ascii_composer/switch_key/Shift_L\": {switch_value}"),
        format!("  \"ascii_composer/switch_key/Shift_R\": {switch_value}"),
    ]);

    let mut bindings: Vec<String> = Vec::new();
    let arrow_paging = config.paging_keys == "arrow_keys";
    let left_right_nav = config.navigation_keys == "left_right";

    if arrow_paging && left_right_nav {
        bindings.push("    - {when: paging, accept: Up, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Down, send: Page_Down}".to_string());
        bindings.push("    - {when: has_menu, accept: Left, send: Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Right, send: Down}".to_string());
    } else if arrow_paging {
        bindings.push("    - {when: paging, accept: Up, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Down, send: Page_Down}".to_string());
    } else if left_right_nav {
        bindings.push("    - {when: has_menu, accept: Left, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: Right, send: Page_Down}".to_string());
    } else if config.paging_keys == "minus_equal" {
        bindings.push("    - {when: paging, accept: minus, send: Page_Up}".to_string());
        bindings.push("    - {when: has_menu, accept: equal, send: Page_Down}".to_string());
    }

    if !bindings.is_empty() {
        default_contents.push("  \"key_binder/bindings\":".to_string());
        default_contents.extend(bindings);
    }
    default_contents.push(String::new());
    default_contents.join("\n")
}

fn save_active_schema_list_sync(schema_ids: Vec<String>) -> Result<QuickSettingsConfig, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let safe_schema_ids = sanitize_schema_ids(schema_ids);
    if safe_schema_ids.is_empty() {
        return Err("至少需要启用一个输入方案".to_string());
    }
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    let mut config = get_quick_settings_sync()?;
    config.schema_id = safe_schema_ids[0].clone();
    write_text_file(
        &user_dir.join("default.custom.yaml"),
        &render_default_custom_with_schema_list(&config, &safe_schema_ids),
        "写入 default.custom.yaml 失败",
    )?;

    get_quick_settings_sync()
}

fn set_active_schema_sync(schema_id: String) -> Result<QuickSettingsConfig, String> {
    let safe_id = sanitize_schema_id(&schema_id);
    if safe_id.is_empty() {
        return Err("方案 ID 不能为空".to_string());
    }

    let user_dir = rime_user_dir()?;
    let mut schema_ids = parse_schema_list(&read_to_string(&user_dir.join("default.custom.yaml")));
    schema_ids.retain(|id| id != &safe_id);
    schema_ids.insert(0, safe_id);
    save_active_schema_list_sync(schema_ids)
}

fn validate_schema_path(path: String) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() || !path.is_file() {
        return Err("方案文件不存在".to_string());
    }

    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return Err("方案文件名无效".to_string());
    };
    if !name.ends_with(".schema.yaml") && !name.ends_with(".custom.yaml") {
        return Err("只能打开 Rime 方案文件".to_string());
    }

    Ok(path)
}

fn open_schema_file_sync(path: String) -> Result<(), String> {
    let path = validate_schema_path(path)?;
    reveal_in_explorer(&path)
}

fn open_schema_dir_sync(path: String) -> Result<(), String> {
    let path = validate_schema_path(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| "方案文件目录无效".to_string())?;
    open_in_explorer(parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_schema_list_from_yaml_patch() {
        let contents = r#"
patch:
  schema_list:
    - schema: rime_ice
    - schema: luna_pinyin
"#;

        assert_eq!(
            parse_schema_list(contents),
            vec!["rime_ice".to_string(), "luna_pinyin".to_string()]
        );
        assert_eq!(parse_schema(contents), Some("rime_ice".to_string()));
    }

    #[test]
    fn parses_patch_values_from_yaml() {
        let contents = r#"
patch:
  "menu/page_size": 9
  "style/horizontal": false
  "style/color_scheme": rime_studio_blue
"#;

        assert_eq!(parse_u32_after_key(contents, "menu/page_size"), Some(9));
        assert_eq!(
            parse_bool_after_key(contents, "style/horizontal"),
            Some(false)
        );
        assert_eq!(
            parse_string_after_key(contents, "style/color_scheme"),
            Some("rime_studio_blue".to_string())
        );
    }

    #[test]
    fn limits_managed_backup_file_names() {
        assert!(is_managed_config_file("default.custom.yaml"));
        assert!(is_managed_config_file("sogou_ext.dict.yaml"));
        assert!(is_managed_config_file("custom_phrase.txt"));
        assert!(!is_managed_config_file("installer.exe"));
        assert!(!is_managed_config_file("notes.txt"));
    }

    #[test]
    fn reads_backup_kind_from_new_and_legacy_names() {
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-manual-123"),
            "manual"
        );
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-before-save-123"),
            "before-save"
        );
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-before-restore-123"),
            "before-restore"
        );
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-before-install-123"),
            "before-install"
        );
        assert_eq!(backup_kind_from_name("backup-rime-studio-123"), "manual");
    }

    #[test]
    fn creates_unique_backup_dir_names() {
        let dir = env::temp_dir().join(format!(
            "rime-studio-backup-kind-test-{}-{}",
            process::id(),
            timestamp()
        ));
        fs::create_dir_all(&dir).expect("create test dir");

        let first =
            create_unique_backup_dir(&dir, BackupKind::BeforeSave).expect("first backup dir");
        let second =
            create_unique_backup_dir(&dir, BackupKind::BeforeSave).expect("second backup dir");

        assert_ne!(first, second);
        assert!(first
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .starts_with("backup-rime-studio-before-save-"));
        assert!(second
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .starts_with("backup-rime-studio-before-save-"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn write_text_file_replaces_existing_contents() {
        let dir = env::temp_dir().join(format!(
            "rime-studio-test-{}-{}",
            process::id(),
            timestamp()
        ));
        let path = dir.join("default.custom.yaml");

        write_text_file(&path, "old", "测试写入").expect("initial write");
        write_text_file(&path, "new", "测试写入").expect("replacement write");

        assert_eq!(fs::read_to_string(&path).expect("read result"), "new");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn parses_tab_separated_dictionary_entries() {
        let contents = "深度学习\tshen du xue xi\t10\n# comment\n空行\tkong hang\n";
        let (entries, skipped) = parse_text_dictionary_entries(contents);

        assert_eq!(skipped, 0);
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0],
            ("深度学习".to_string(), "shen du xue xi".to_string(), 10)
        );
        assert_eq!(entries[1], ("空行".to_string(), "kong hang".to_string(), 1));
    }

    #[test]
    fn removes_duplicate_dictionary_entry_lines_only() {
        let contents = [
            "# Imported by Rime Studio.",
            "---",
            "name: custom",
            "sort: by_weight",
            "...",
            "# duplicated comments should stay",
            "# duplicated comments should stay",
            "深度学习\tshen du xue xi\t10",
            "深度学习\tshen du xue xi\t10",
            "机器学习\tji qi xue xi\t8",
            "",
        ]
        .join("\n");

        let (cleaned, removed) = remove_duplicate_dictionary_lines(&contents);

        assert_eq!(removed, 1);
        assert_eq!(
            cleaned.matches("# duplicated comments should stay").count(),
            2
        );
        assert_eq!(cleaned.matches("深度学习\tshen du xue xi\t10").count(), 1);
        assert!(cleaned.contains("机器学习\tji qi xue xi\t8"));
    }

    #[test]
    fn sanitizes_dictionary_file_names() {
        assert_eq!(sanitize_dict_file_name("sogou.scel"), "sogou.dict.yaml");
        assert_eq!(
            sanitize_dict_file_name("custom.dict.yaml"),
            "custom.dict.yaml"
        );
        assert_eq!(
            sanitize_dict_file_name("../bad name.txt"),
            "bad_name.dict.yaml"
        );
    }

    #[test]
    fn parses_sogou_bin_record() {
        let mut data = b"SGPU".to_vec();
        data.resize(32, 0);
        data.extend([0, 1, 0, 3]);
        data.extend(6u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(173u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(16u16.to_le_bytes());
        data.extend(6u16.to_le_bytes());
        data.extend("阿里啊".encode_utf16().flat_map(u16::to_le_bytes));
        data.extend(6u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(173u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());

        let (entries, skipped) = parse_sogou_bin_entries(&data).expect("parse sogou bin");
        assert_eq!(skipped, 0);
        assert_eq!(
            entries,
            vec![("阿里啊".to_string(), "a li a".to_string(), 1)]
        );
    }
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
async fn get_quick_settings() -> Result<QuickSettingsConfig, String> {
    run_blocking(get_quick_settings_sync).await
}

#[tauri::command]
async fn save_quick_settings(config: QuickSettingsConfig) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || save_quick_settings_sync(config)).await
}

#[tauri::command]
async fn inspect_config_health() -> Result<ConfigHealthReport, String> {
    run_blocking(inspect_config_health_sync).await
}

#[tauri::command]
async fn repair_config_health() -> Result<ConfigHealthReport, String> {
    run_blocking(repair_config_health_sync).await
}

#[tauri::command]
async fn get_rime_ice_settings() -> Result<RimeIceSettings, String> {
    run_blocking(get_rime_ice_settings_sync).await
}

#[tauri::command]
async fn save_rime_ice_settings(settings: RimeIceSettings) -> Result<RimeIceSettings, String> {
    run_blocking(move || save_rime_ice_settings_sync(settings)).await
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
async fn open_config_file(name: String) -> Result<(), String> {
    run_blocking(move || open_config_file_sync(name)).await
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
async fn delete_backup(backup_name: String) -> Result<(), String> {
    run_blocking(move || delete_backup_sync(backup_name)).await
}

#[tauri::command]
async fn delete_dictionary(dict_name: String) -> Result<(), String> {
    run_blocking(move || delete_dictionary_sync(dict_name)).await
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
async fn clean_dictionary_duplicates(dict_name: String) -> Result<DictionaryCleanResult, String> {
    run_blocking(move || clean_dictionary_duplicates_sync(dict_name)).await
}

#[tauri::command]
async fn get_dictionary_config() -> Result<DictionaryConfig, String> {
    run_blocking(read_dictionary_config_sync).await
}

#[tauri::command]
async fn add_dictionary_to_current_schema(reference: String) -> Result<DictionaryConfig, String> {
    run_blocking(move || add_dictionary_to_current_schema_sync(reference)).await
}

#[tauri::command]
async fn remove_dictionary_from_current_schema(
    reference: String,
) -> Result<DictionaryConfig, String> {
    run_blocking(move || remove_dictionary_from_current_schema_sync(reference)).await
}

#[tauri::command]
async fn save_dictionary_imports(imports: Vec<String>) -> Result<DictionaryConfig, String> {
    run_blocking(move || save_dictionary_imports_sync(imports)).await
}

#[tauri::command]
async fn import_dictionary(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_dictionary_sync(source_name, data)).await
}

#[tauri::command]
async fn export_dictionary(dict_name: String) -> Result<DictionaryExportResult, String> {
    run_blocking(move || export_dictionary_sync(dict_name)).await
}

#[tauri::command]
async fn download_rime_installer() -> Result<RimeDownloadResult, String> {
    run_blocking(download_rime_installer_sync).await
}

#[tauri::command]
async fn launch_rime_installer(path: String) -> Result<(), String> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[tauri::command]
async fn list_schemas() -> Result<Vec<SchemaInfo>, String> {
    run_blocking(list_schemas_sync).await
}

#[tauri::command]
async fn copy_schema(schema_id: String) -> Result<String, String> {
    run_blocking(move || copy_schema_sync(schema_id)).await
}

#[tauri::command]
async fn set_active_schema(schema_id: String) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || set_active_schema_sync(schema_id)).await
}

#[tauri::command]
async fn save_active_schema_list(schema_ids: Vec<String>) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || save_active_schema_list_sync(schema_ids)).await
}

#[tauri::command]
async fn open_schema_file(path: String) -> Result<(), String> {
    run_blocking(move || open_schema_file_sync(path)).await
}

#[tauri::command]
async fn open_schema_dir(path: String) -> Result<(), String> {
    run_blocking(move || open_schema_dir_sync(path)).await
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
            get_quick_settings,
            save_quick_settings,
            inspect_config_health,
            repair_config_health,
            get_rime_ice_settings,
            save_rime_ice_settings,
            save_appearance_config,
            list_backups,
            create_backup,
            open_rime_user_dir,
            open_config_file,
            open_plum_dir,
            open_backup_dir,
            restore_backup,
            delete_backup,
            delete_dictionary,
            get_custom_phrases,
            save_custom_phrases,
            list_dictionaries,
            get_dict_health,
            clean_dictionary_duplicates,
            get_dictionary_config,
            add_dictionary_to_current_schema,
            remove_dictionary_from_current_schema,
            save_dictionary_imports,
            import_dictionary,
            export_dictionary,
            download_rime_installer,
            launch_rime_installer,
            list_schemas,
            copy_schema,
            set_active_schema,
            save_active_schema_list,
            open_schema_file,
            open_schema_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
