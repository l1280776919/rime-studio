use crate::backend::*;
use crate::types::RimeError;
use serde::Serialize;
use serde_yaml::Value;
use std::{
    collections::{HashSet, VecDeque},
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub(crate) struct ConfigDiagnostic {
    pub filename: String,
    pub editable: bool,
    pub code: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct DiagnosticReport {
    pub app_version: String,
    pub platform: String,
    pub architecture: String,
    pub candidates: Vec<DeployerCandidate>,
    pub issues: Vec<ConfigDiagnostic>,
    pub files_checked: usize,
}

/// Only parse dictionary headers; vocabulary rows can be very large and private.
pub(crate) fn read_yaml_header(path: &Path, dictionary: bool) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|_| "unreadable".to_string())?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    loop {
        // Limit each read as well as the total, including files with no newlines.
        let mut line = String::new();
        let count = std::io::Read::take(&mut reader, 2 * 1024 * 1024 + 1)
            .read_line(&mut line)
            .map_err(|_| "unreadable".to_string())?;
        if contents.len() + count > 2 * 1024 * 1024 {
            return Err("too_large".into());
        }
        if count == 0 {
            break;
        }
        if dictionary && line.trim() == "..." {
            break;
        }
        contents.push_str(&line);
    }
    Ok(contents)
}

/// Respect user overrides and reuse the schema browser's shared source directories.
fn diagnostic_path(
    user_dir: &Path,
    system_dirs: &[PathBuf],
    filename: &str,
) -> Result<(PathBuf, bool), RimeError> {
    let user = resolve_user_relative_path(user_dir, filename, false)?;
    if user.exists() || filename.ends_with(".custom.yaml") {
        return Ok((user, true));
    }
    for root in system_dirs.iter().filter(|root| root.exists()) {
        if let Ok(path) = resolve_user_relative_path(root, filename, true) {
            return Ok((path, false));
        }
    }
    Ok((user, true))
}

/// Follow the selected schemas and their YAML dependencies, never build/ outputs.
/// Limits prevent cyclic references or oversized files from blocking deployment.
pub(crate) fn inspect_config_diagnostics(user_dir: &Path) -> (Vec<ConfigDiagnostic>, usize) {
    inspect_config_diagnostics_in(user_dir, &system_data_dirs())
}

fn inspect_config_diagnostics_in(
    user_dir: &Path,
    system_dirs: &[PathBuf],
) -> (Vec<ConfigDiagnostic>, usize) {
    if !user_dir.exists() {
        return (
            vec![ConfigDiagnostic {
                filename: "Rime 用户目录".into(),
                editable: false,
                code: "missing_user_dir".into(),
                message: "用户目录尚未创建，请先启动小狼毫后重新检查".into(),
                line: None,
                column: None,
            }],
            0,
        );
    }
    let mut queue: VecDeque<(String, bool)> = [
        "default.custom.yaml",
        "weasel.custom.yaml",
        "rime_ice.custom.yaml",
    ]
    .into_iter()
    .map(|name| (name.into(), false))
    .collect();
    // Use the base menu only when no user override is present.
    let custom = read_yaml_header(&user_dir.join("default.custom.yaml"), false).unwrap_or_default();
    if yaml_lookup(&custom, "schema_list").is_none() {
        queue.push_back(("default.yaml".into(), false));
    }
    let mut visited = HashSet::new();
    let mut issues = Vec::new();
    let mut checked = 0;
    while let Some((filename, required)) = queue.pop_front() {
        if !visited.insert(filename.clone()) {
            continue;
        }
        let resolved = diagnostic_path(user_dir, system_dirs, &filename);
        let editable = resolved.as_ref().is_ok_and(|(_, editable)| *editable);
        let mut issue = |code: &str, message: &str, line: Option<usize>, column: Option<usize>| {
            issues.push(ConfigDiagnostic {
                filename: filename.clone(),
                editable,
                code: code.into(),
                message: message.into(),
                line,
                column,
            });
        };
        if visited.len() > 128 {
            issue("limit", "依赖超过 128 个文件，剩余文件未检查", None, None);
            break;
        }
        let path = match resolved {
            Ok((path, _)) => path,
            Err(_) => {
                issue("invalid_path", "配置路径不可访问或越出用户目录", None, None);
                continue;
            }
        };
        if !path.exists() {
            if required {
                issue(
                    "missing",
                    "启用方案引用的配置文件不存在，请安装或修复对应方案",
                    None,
                    None,
                );
            }
            continue;
        }
        let content = match read_yaml_header(&path, filename.ends_with(".dict.yaml")) {
            Ok(content) => content,
            Err(code) => {
                issue(
                    &code,
                    "文件无法读取或 YAML 头超过 2 MiB，未完成检查",
                    None,
                    None,
                );
                continue;
            }
        };
        checked += 1;
        let yaml: Value = match serde_yaml::from_str(&content) {
            Ok(yaml) => yaml,
            Err(error) => {
                let location = error.location();
                issue(
                    "yaml_parse",
                    "YAML 语法错误，请检查缩进、冒号和引号",
                    location.as_ref().map(|v| v.line()),
                    location.as_ref().map(|v| v.column()),
                );
                continue;
            }
        };
        queue.extend(config_yaml_dependencies(&filename, &content, &yaml));
    }
    (issues, checked)
}

/// Shared static YAML references used by environment checks and migration previews.
pub(crate) fn config_yaml_dependencies(
    filename: &str,
    content: &str,
    yaml: &Value,
) -> Vec<(String, bool)> {
    let mut dependencies = Vec::new();
    if filename == "default.custom.yaml" || filename == "default.yaml" {
        for schema in parse_schema_list(content) {
            dependencies.push((format!("{schema}.schema.yaml"), true));
        }
    }
    if filename.ends_with(".schema.yaml") {
        let id = filename.trim_end_matches(".schema.yaml");
        dependencies.push((format!("{id}.custom.yaml"), false));
        if let Some(deps) = yaml
            .get("schema")
            .and_then(|v| v.get("dependencies"))
            .and_then(Value::as_sequence)
        {
            for dep in deps.iter().filter_map(Value::as_str) {
                dependencies.push((format!("{dep}.schema.yaml"), true));
            }
        }
        // Translators can have custom names, so inspect every top-level section.
        if let Some(sections) = yaml.as_mapping() {
            for section in sections.values() {
                if let Some(dict) = section
                    .get("dictionary")
                    .and_then(Value::as_str)
                    .filter(|v| !v.is_empty())
                {
                    dependencies.push((format!("{dict}.dict.yaml"), true));
                }
            }
        }
    }
    if let Some(imports) = yaml.get("import_tables").and_then(Value::as_sequence) {
        for name in imports.iter().filter_map(Value::as_str) {
            dependencies.push((format!("{name}.dict.yaml"), true));
        }
    }
    dependencies
}

pub(crate) fn get_diagnostic_report_sync() -> Result<DiagnosticReport, RimeError> {
    let user_dir = rime_user_dir()?;
    let (issues, files_checked) = inspect_config_diagnostics(&user_dir);
    Ok(DiagnosticReport {
        app_version: env!("CARGO_PKG_VERSION").into(),
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        candidates: discover_deployer_candidates(),
        issues,
        files_checked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn follows_schema_dependencies_uses_shared_sources_and_ignores_dictionary_rows(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("rime-diagnostics-{}", std::process::id()));
        let user = root.join("user");
        let shared = root.join("shared");
        fs::create_dir_all(&user)?;
        fs::create_dir_all(&shared)?;
        fs::write(
            user.join("default.custom.yaml"),
            "patch:\n  schema_list:\n    - schema: demo\n",
        )?;
        fs::write(
            shared.join("demo.schema.yaml"),
            "schema:\n  dependencies: [dependency]\ntranslator:\n  dictionary: demo\n",
        )?;
        fs::write(
            shared.join("demo.dict.yaml"),
            "---\nname: demo\nimport_tables: [missing]\n...\nprivate words\t[[[not yaml\n",
        )?;
        fs::write(user.join("dependency.schema.yaml"), "schema: [\n")?;
        let (issues, checked) = inspect_config_diagnostics_in(&user, &[shared]);
        assert_eq!(checked, 4);
        assert_eq!(issues.len(), 2);
        assert!(issues
            .iter()
            .any(|issue| issue.filename == "dependency.schema.yaml"
                && issue.code == "yaml_parse"
                && issue.editable
                && issue.line.is_some()));
        assert!(issues
            .iter()
            .any(|issue| issue.filename == "missing.dict.yaml" && issue.code == "missing"));
        assert!(issues
            .iter()
            .all(|issue| !issue.message.contains("private")));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
