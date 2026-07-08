use crate::*;
use std::{env, ffi::OsStr, fs, path::{Path, PathBuf}, process::Command};
use serde_yaml::{Mapping, Value};

pub(crate) fn rime_user_dir() -> Result<PathBuf, String> {
    let appdata = env::var("APPDATA").map_err(|_| "APPDATA 环境变量不可用".to_string())?;
    Ok(PathBuf::from(appdata).join("Rime"))
}

pub(crate) fn app_data_dir() -> Result<PathBuf, String> {
    let local_appdata =
        env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA 环境变量不可用".to_string())?;
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

pub(crate) fn weasel_deployers_under(root: &Path) -> Vec<PathBuf> {
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

pub(crate) fn command_success(path: &Path, arg: &str) -> bool {
    let mut command = Command::new(path);
    command.arg(arg);
    suppress_console_window(&mut command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(crate) fn command_path_success(command: &str, arg: &str) -> bool {
    let mut command = Command::new(command);
    command.arg(arg);
    suppress_console_window(&mut command)
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

pub(crate) fn locate_git_bash() -> Option<PathBuf> {
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

