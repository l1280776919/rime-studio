use crate::*;
use serde_yaml::{Mapping, Value};
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

static CONFIG_WRITES: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn lock_config_write() -> Result<std::sync::MutexGuard<'static, ()>, RimeError> {
    CONFIG_WRITES
        .lock()
        .map_err(|_| RimeError::FileOperationError("配置写入锁不可用".into()))
}

pub(crate) fn rime_user_dir() -> Result<PathBuf, RimeError> {
    let appdata = env::var("APPDATA")
        .map_err(|_| RimeError::EnvVarNotFound("APPDATA 环境变量不可用".to_string()))?;
    Ok(PathBuf::from(appdata).join("Rime"))
}

pub(crate) fn app_data_dir() -> Result<PathBuf, RimeError> {
    let local_appdata = env::var("LOCALAPPDATA")
        .map_err(|_| RimeError::EnvVarNotFound("LOCALAPPDATA 环境变量不可用".to_string()))?;
    Ok(PathBuf::from(local_appdata).join("RimeStudio"))
}

pub(crate) fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

// Missing custom files may be created, but unreadable existing files must never
// be treated as empty during a read-modify-write operation.
pub(crate) fn read_optional_config(path: &Path) -> Result<String, RimeError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(RimeError::FileOperationError(format!(
            "读取 {} 失败，已中止写入: {err}",
            path.display()
        ))),
    }
}

pub(crate) fn read_user_custom_config(user_dir: &Path, name: &str) -> Result<String, RimeError> {
    if !user_dir.exists() {
        return Ok(String::new());
    }
    let path = resolve_user_relative_path(user_dir, name, false)?;
    let contents = read_optional_config(&path)?;
    crate::backend::merge_custom_yaml(&contents, |_| {})?;
    Ok(contents)
}

pub(crate) fn yaml_mapping_get<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

pub(crate) fn yaml_path_get<'a>(value: &'a Value, key_path: &str) -> Option<&'a Value> {
    let mut current = value;
    for key in key_path.split('/') {
        let Value::Mapping(mapping) = current else {
            return None;
        };
        current = yaml_mapping_get(mapping, key)?;
    }
    Some(current)
}

pub(crate) fn yaml_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

pub(crate) fn yaml_lookup(contents: &str, key: &str) -> Option<Value> {
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

pub(crate) fn suppress_console_window(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

pub(crate) fn join_user_rel(user_dir: &Path, rel: &str) -> PathBuf {
    let mut path = user_dir.to_path_buf();
    for part in rel.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        path.push(part);
    }
    path
}

/// Validate Windows-relative paths even when regression tests run on Linux.
pub(crate) fn valid_user_relative_path(relative: &str) -> bool {
    !relative.is_empty()
        && relative.split('/').all(|part| {
            let stem = part
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with(['.', ' '])
                && !part
                    .chars()
                    .any(|ch| ch.is_control() || "\\:<>\"|?*".contains(ch))
                && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !((stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.len() == 4
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}

pub(crate) fn resolve_user_relative_path(
    user_dir: &Path,
    relative: &str,
    must_exist: bool,
) -> Result<PathBuf, RimeError> {
    if !valid_user_relative_path(relative) {
        return Err(RimeError::FileOperationError(
            "用户目录相对路径无效".to_string(),
        ));
    }
    let root = fs::canonicalize(user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("读取用户目录失败: {err}")))?;
    let path = root.join(relative);
    // Check the nearest existing ancestor, including the target itself. This
    // covers junctions/symlinks followed by not-yet-created subdirectories.
    let mut ancestor = path.as_path();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound && !must_exist => {
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| RimeError::FileOperationError("路径无效".to_string()))?;
            }
            Err(err) => {
                return Err(RimeError::FileOperationError(format!(
                    "读取路径失败: {err}"
                )))
            }
        }
    }
    let canonical = fs::canonicalize(ancestor)
        .map_err(|err| RimeError::FileOperationError(format!("解析路径失败: {err}")))?;
    if !canonical.starts_with(&root) {
        return Err(RimeError::FileOperationError(
            "路径越出 Rime 用户目录，已中止操作".to_string(),
        ));
    }
    Ok(if must_exist { canonical } else { path })
}

pub(crate) fn file_status(user_dir: &Path, name: &str) -> FileStatus {
    let path = join_user_rel(user_dir, name);
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

pub(crate) fn parse_schema(default_custom: &str) -> Option<String> {
    parse_schema_list(default_custom).into_iter().next()
}

pub(crate) fn parse_schema_list(default_custom: &str) -> Vec<String> {
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
        return schemas;
    }

    Vec::new()
}

pub(crate) fn parse_u32_after_key(contents: &str, key: &str) -> Option<u32> {
    yaml_lookup(contents, key)
        .and_then(|value| yaml_value_to_string(&value))
        .and_then(|value| value.parse::<u32>().ok())
}

pub(crate) fn parse_quoted_value(contents: &str, key: &str) -> Option<String> {
    yaml_lookup(contents, key).and_then(|value| yaml_value_to_string(&value))
}

pub(crate) fn parse_bool_after_key(contents: &str, key: &str) -> Option<bool> {
    match yaml_lookup(contents, key)? {
        Value::Bool(value) => Some(value),
        Value::String(value) => match value.as_str() {
            "true" | "True" | "yes" => Some(true),
            "false" | "False" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn parse_string_after_key(contents: &str, key: &str) -> Option<String> {
    yaml_lookup(contents, key)
        .and_then(|value| yaml_value_to_string(&value))
        .filter(|value| !value.is_empty())
}

pub(crate) fn normalize_color(value: Option<String>, fallback: &str) -> String {
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

#[cfg(target_os = "windows")]
pub(crate) fn weasel_root_from_registry() -> Vec<PathBuf> {
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    };
    use winreg::RegKey;

    let mut roots = Vec::new();
    // NSIS writes InstallDir to the 32-bit view, even on 64-bit Windows.
    // Read both views explicitly instead of depending on our process bitness.
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for view in [KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
            if let Ok(key) = RegKey::predef(hive)
                .open_subkey_with_flags("Software\\Rime\\Weasel", KEY_READ | view)
            {
                roots.extend(weasel_paths_from_key(&key));
            }
        }
    }
    roots
}

#[cfg(target_os = "windows")]
fn weasel_paths_from_key(key: &winreg::RegKey) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = [
        "WeaselRoot",
        "InstallDir",
        "WeaselRootPath",
        "WeaselDeployer",
    ]
    .into_iter()
    .filter_map(|name| key.get_value::<String, _>(name).ok())
    .filter_map(|value| weasel_registry_path(&value))
    .collect();
    if let Ok(value) = key.get_value::<String, _>("Execute") {
        if let Some(path) = weasel_registry_path(&value) {
            if let Some(parent) = path.parent() {
                roots.push(parent.to_path_buf());
            }
        }
    }
    roots
}

fn weasel_registry_path(value: &str) -> Option<PathBuf> {
    let value = value.trim().trim_matches('"');
    (!value.is_empty()).then(|| PathBuf::from(value))
}

pub(crate) fn validate_weasel_deployer(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.eq_ignore_ascii_case("WeaselDeployer.exe"))
}

pub(crate) fn set_weasel_deployer_sync(path: String) -> Result<String, RimeError> {
    let path = weasel_registry_path(&path)
        .filter(|path| path.is_absolute() && validate_weasel_deployer(path))
        .ok_or_else(|| {
            RimeError::DeployerNotFound(
                "请选择已安装的小狼毫目录中的 WeaselDeployer.exe 完整路径".to_string(),
            )
        })?;
    let path = path
        .canonicalize()
        .map_err(|err| RimeError::FileOperationError(format!("读取小狼毫路径失败: {err}")))?;
    let app_dir = app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建应用数据目录失败: {err}")))?;
    let display_path = path.display().to_string();
    fs::write(app_dir.join("weasel-deployer.txt"), &display_path)
        .map_err(|err| RimeError::FileOperationError(format!("保存小狼毫路径失败: {err}")))?;
    Ok(display_path)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn weasel_root_from_registry() -> Vec<PathBuf> {
    Vec::new()
}

pub(crate) fn weasel_deployers_under(root: &Path) -> Vec<PathBuf> {
    if validate_weasel_deployer(root) {
        return vec![root.to_path_buf()];
    }
    if !root.exists() {
        return Vec::new();
    }

    let mut deployers = Vec::new();
    let direct_deployer = root.join("WeaselDeployer.exe");
    if direct_deployer.is_file() {
        deployers.push(direct_deployer);
    }

    let mut subdirs: Vec<PathBuf> = fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .map(|name| name.to_ascii_lowercase().starts_with("weasel"))
                    .unwrap_or(false)
        })
        .collect();

    // Sort subdirectories descending so newer version (e.g. weasel-0.18 > weasel-0.17) is preferred
    subdirs.sort_by(|a, b| b.cmp(a));

    for dir in subdirs {
        let deployer = dir.join("WeaselDeployer.exe");
        if deployer.is_file() {
            deployers.push(deployer);
        }
    }

    deployers
}

pub(crate) fn resolve_windows_shortcut(path: &Path) -> Option<PathBuf> {
    if path.extension().and_then(OsStr::to_str) != Some("lnk") {
        return Some(path.to_path_buf());
    }

    let script = format!(
        "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; $s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}'); $s.TargetPath",
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

/// Paths are shown locally; the exported report excludes them.
#[derive(serde::Serialize)]
pub(crate) struct DeployerCandidate {
    pub source: String,
    pub path: String,
    pub valid: bool,
    pub reason: String,
}

pub(crate) fn discover_deployer_candidates() -> Vec<DeployerCandidate> {
    discover_deployer_candidates_internal(false)
}

/// Normal scans stop resolving shortcuts after a match; diagnostics inspect every source.
fn discover_deployer_candidates_internal(stop_after_match: bool) -> Vec<DeployerCandidate> {
    let mut candidates: Vec<(String, PathBuf)> = Vec::new();
    let mut add_roots = |source: &str, root: &Path| {
        let found = weasel_deployers_under(root);
        if found.is_empty() {
            candidates.push((source.into(), root.to_path_buf()));
        }
        candidates.extend(found.into_iter().map(|path| (source.into(), path)));
    };

    // 1. Check registry-discovered paths
    for root in weasel_root_from_registry() {
        add_roots("注册表", &root);
    }

    // 2. Check standard Program Files installation paths
    let mut rime_parents = vec![
        PathBuf::from(r"C:\Program Files\Rime"),
        PathBuf::from(r"C:\Program Files (x86)\Rime"),
    ];
    if let Ok(pf) = env::var("ProgramFiles") {
        rime_parents.push(PathBuf::from(pf).join("Rime"));
    }
    if let Ok(pf86) = env::var("ProgramFiles(x86)") {
        rime_parents.push(PathBuf::from(pf86).join("Rime"));
    }
    if let Ok(pf_w64) = env::var("ProgramW6432") {
        rime_parents.push(PathBuf::from(pf_w64).join("Rime"));
    }

    for parent in rime_parents {
        add_roots("安装目录", &parent);
    }

    // 3. Machine-wide and per-user shortcuts in all installer languages.
    for base in ["PROGRAMDATA", "APPDATA"] {
        if let Ok(base) = env::var(base) {
            candidates.extend(
                weasel_shortcuts_under(
                    &PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"),
                )
                .into_iter()
                .map(|path| ("开始菜单快捷方式".into(), path)),
            );
        }
    }

    if let Ok(app_dir) = app_data_dir() {
        let saved = read_to_string(&app_dir.join("weasel-deployer.txt"));
        if let Some(path) = weasel_registry_path(&saved) {
            candidates.insert(0, ("手动指定".into(), path));
        }
    }
    candidates
        .into_iter()
        .scan(false, |found, (source, path)| {
            if stop_after_match && *found {
                return None;
            }
            let target = resolve_windows_shortcut(&path);
            let valid = target
                .as_ref()
                .is_some_and(|path| validate_weasel_deployer(path));
            let reason = if valid {
                "可用"
            } else if !path.exists() {
                "路径不存在"
            } else {
                "未发现有效部署器或快捷方式目标无效"
            };
            *found = valid;
            Some(DeployerCandidate {
                source,
                path: target.unwrap_or(path).to_string_lossy().into_owned(),
                valid,
                reason: reason.into(),
            })
        })
        .collect()
}

pub(crate) fn locate_deployer() -> Option<PathBuf> {
    discover_deployer_candidates_internal(true)
        .into_iter()
        .find(|entry| entry.valid)
        .map(|entry| PathBuf::from(entry.path))
}

fn weasel_shortcuts_under(programs: &Path) -> Vec<PathBuf> {
    [
        ("小狼毫输入法", "【小狼毫】重新部署.lnk"),
        ("小狼毫輸入法", "【小狼毫】重新部署.lnk"),
        ("Weasel", "Weasel Deploy.lnk"),
    ]
    .into_iter()
    .map(|(folder, name)| programs.join(folder).join(name))
    .collect()
}

pub(crate) fn locate_weasel_server() -> Option<PathBuf> {
    if let Some(deployer) = locate_deployer() {
        if let Some(parent) = deployer.parent() {
            let candidate = parent.join("WeaselServer.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

pub(crate) fn command_success(command: impl AsRef<Path>, arg: &str) -> bool {
    let mut cmd = Command::new(command.as_ref());
    cmd.arg(arg);
    suppress_console_window(&mut cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(crate) fn locate_from_where(command: &str) -> Vec<PathBuf> {
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

pub(crate) fn git_roots_from_path(path: &Path) -> Vec<PathBuf> {
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

pub(crate) fn locate_git() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from(r"C:\Program Files\Git\cmd\git.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Git\cmd\git.exe"),
    ];

    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local_appdata)
                .join("Programs")
                .join("Git")
                .join("cmd")
                .join("git.exe"),
        );
    }
    if let Ok(pf) = env::var("ProgramFiles") {
        candidates.push(PathBuf::from(pf).join("Git").join("cmd").join("git.exe"));
    }

    candidates.extend(locate_from_where("git.exe"));
    candidates.extend(locate_from_where("git"));

    candidates
        .into_iter()
        .find(|path| path.exists() && command_success(path, "--version"))
        .or_else(|| {
            if command_success(Path::new("git"), "--version") {
                Some(PathBuf::from("git"))
            } else {
                None
            }
        })
}

pub(crate) fn locate_git_bash() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from(r"C:\Program Files\Git\bin\bash.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Git\bin\bash.exe"),
    ];

    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let git_dir = PathBuf::from(&local_appdata).join("Programs").join("Git");
        candidates.push(git_dir.join("bin").join("bash.exe"));
        candidates.push(git_dir.join("usr").join("bin").join("bash.exe"));
    }

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

#[cfg(test)]
mod weasel_detection_tests {
    use super::*;

    #[test]
    fn optional_config_distinguishes_missing_from_unreadable() {
        let root = env::temp_dir().join(format!("rime-config-read-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create fixture");
        let path = root.join("test.yaml");
        assert_eq!(read_optional_config(&path).expect("missing file"), "");
        fs::write(&path, [0xff]).expect("write invalid UTF-8");
        assert!(read_optional_config(&path).is_err());
        assert_eq!(fs::read(&path).expect("original retained"), vec![0xff]);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_custom_install_dir_and_direct_deployer() {
        let root = env::temp_dir().join(format!("rime-studio-weasel-test-{}", std::process::id()));
        let version_dir = root.join("中文 自定义安装").join("Weasel-0.17.4");
        fs::create_dir_all(&version_dir).expect("create installation");
        let deployer = version_dir.join("WeaselDeployer.exe");
        fs::write(&deployer, b"MZ").expect("write deployer fixture");
        assert_eq!(
            weasel_deployers_under(&root.join("中文 自定义安装")),
            vec![deployer.clone()]
        );
        assert_eq!(weasel_deployers_under(&version_dir), vec![deployer.clone()]);
        assert_eq!(weasel_deployers_under(&deployer), vec![deployer.clone()]);
        assert!(validate_weasel_deployer(&deployer));
        assert!(!validate_weasel_deployer(&version_dir));
        fs::remove_dir_all(&root).expect("remove fixture");
        assert!(!validate_weasel_deployer(&deployer));
    }

    #[test]
    fn handles_quoted_registry_values_and_all_shortcut_languages() {
        assert_eq!(
            weasel_registry_path("  \"D:\\我的输入法\\weasel-0.17.4\"  "),
            Some(PathBuf::from(r"D:\我的输入法\weasel-0.17.4"))
        );
        assert!(weasel_registry_path("  ").is_none());
        let programs = Path::new("Programs");
        let shortcuts = weasel_shortcuts_under(programs);
        assert!(shortcuts.contains(&programs.join("小狼毫输入法").join("【小狼毫】重新部署.lnk")));
        assert!(shortcuts.contains(&programs.join("小狼毫輸入法").join("【小狼毫】重新部署.lnk")));
        assert!(shortcuts.contains(&programs.join("Weasel").join("Weasel Deploy.lnk")));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn reads_official_installer_registry_fields() {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        let hive = RegKey::predef(HKEY_CURRENT_USER);
        let key_path = format!("Software\\RimeStudio\\DetectionTest-{}", std::process::id());
        let (key, _) = hive
            .create_subkey(&key_path)
            .expect("create isolated test key");
        key.set_value("InstallDir", &r"D:\输入法\Rime")
            .expect("write official field");
        key.set_value("WeaselRoot", &r"D:\输入法\Rime\weasel-0.17.4")
            .expect("write official field");
        let roots = weasel_paths_from_key(&key);
        hive.delete_subkey_all(&key_path)
            .expect("remove isolated test key");
        assert!(roots.contains(&PathBuf::from(r"D:\输入法\Rime")));
        assert!(roots.contains(&PathBuf::from(r"D:\输入法\Rime\weasel-0.17.4")));
    }
}
