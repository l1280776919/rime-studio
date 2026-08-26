use crate::*;
use serde_yaml::{Mapping, Value};
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

pub(crate) fn file_status(user_dir: &Path, name: &str) -> FileStatus {
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

pub(crate) fn parse_u32_after_key(contents: &str, key: &str) -> Option<u32> {
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

pub(crate) fn parse_quoted_value(contents: &str, key: &str) -> Option<String> {
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

pub(crate) fn parse_bool_after_key(contents: &str, key: &str) -> Option<bool> {
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

pub(crate) fn parse_string_after_key(contents: &str, key: &str) -> Option<String> {
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
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let mut roots = Vec::new();
    let subkeys = [
        "Software\\Rime\\Weasel",
        "Software\\WOW6432Node\\Rime\\Weasel",
    ];

    for key_path in subkeys {
        for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            if let Ok(key) = RegKey::predef(hive).open_subkey(key_path) {
                if let Ok(val) = key.get_value::<String, _>("WeaselRootPath") {
                    let p = PathBuf::from(val.trim());
                    if !p.as_os_str().is_empty() {
                        roots.push(p);
                    }
                }
                if let Ok(val) = key.get_value::<String, _>("WeaselDeployer") {
                    let p = PathBuf::from(val.trim());
                    if p.exists() {
                        roots.push(p);
                    }
                }
                if let Ok(val) = key.get_value::<String, _>("Execute") {
                    let p = PathBuf::from(val.trim());
                    if let Some(parent) = p.parent() {
                        roots.push(parent.to_path_buf());
                    }
                }
            }
        }
    }
    roots
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn weasel_root_from_registry() -> Vec<PathBuf> {
    Vec::new()
}

pub(crate) fn weasel_deployers_under(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    let mut deployers = Vec::new();
    let direct_deployer = root.join("WeaselDeployer.exe");
    if direct_deployer.exists() {
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
                    .map(|name| name.starts_with("weasel"))
                    .unwrap_or(false)
        })
        .collect();

    // Sort subdirectories descending so newer version (e.g. weasel-0.18 > weasel-0.17) is preferred
    subdirs.sort_by(|a, b| b.cmp(a));

    for dir in subdirs {
        let deployer = dir.join("WeaselDeployer.exe");
        if deployer.exists() {
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

pub(crate) fn locate_deployer() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    // 1. Check registry-discovered paths
    for root in weasel_root_from_registry() {
        if root.is_file() && root.file_name().and_then(OsStr::to_str) == Some("WeaselDeployer.exe") {
            candidates.push(root);
        } else {
            candidates.extend(weasel_deployers_under(&root));
        }
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
        candidates.extend(weasel_deployers_under(&parent));
    }

    // 3. Start menu shortcut
    let start_menu_shortcut = PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\小狼毫输入法\【小狼毫】重新部署.lnk",
    );
    if start_menu_shortcut.exists() {
        candidates.push(start_menu_shortcut);
    }

    candidates
        .into_iter()
        .filter(|path| path.exists())
        .find_map(|path| resolve_windows_shortcut(&path))
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
        candidates.push(PathBuf::from(local_appdata).join("Programs").join("Git").join("cmd").join("git.exe"));
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

