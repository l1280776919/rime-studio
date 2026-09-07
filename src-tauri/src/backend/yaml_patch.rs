use crate::*;
use serde_yaml::{Mapping, Value};

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

const HEADER: &str = "# Updated by Rime Studio. Unknown patch keys are preserved.\n# Previous versions are kept in RimeStudio backups.\n";

pub(crate) fn yaml_str(value: &str) -> Value {
    Value::String(value.to_string())
}

pub(crate) fn parse_yaml_mapping(contents: &str) -> Result<Mapping, RimeError> {
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Ok(Mapping::new());
    }

    match serde_yaml::from_str::<Value>(contents) {
        Ok(Value::Mapping(mapping)) => Ok(mapping),
        Ok(Value::Null) => Ok(Mapping::new()),
        Ok(_) => Err(RimeError::YamlParseError(
            "配置顶层必须是映射，已中止写入以免覆盖原文件".to_string(),
        )),
        Err(err) => Err(RimeError::YamlParseError(format!(
            "现有 YAML 无法解析，已中止写入以免覆盖: {err}"
        ))),
    }
}

pub(crate) fn serialize_custom_yaml(root: &Mapping) -> Result<String, RimeError> {
    let body = serde_yaml::to_string(&Value::Mapping(root.clone()))
        .map_err(|err| RimeError::YamlParseError(format!("序列化 YAML 失败: {err}")))?;
    let body = body.trim_start_matches("---").trim_start();
    Ok(format!("{HEADER}\n{body}"))
}

fn mapping_key_string(key: &Value) -> Option<String> {
    match key {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn patches_semantically_eq(left: &Mapping, right: &Mapping) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .all(|(key, value)| right.get(key) == Some(value))
}

fn indent_width(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}

fn parse_mapping_key(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix('"') {
        let end = rest.find('"')?;
        let after = rest.get(end + 1..)?.trim_start();
        if after.starts_with(':') {
            return Some(rest[..end].to_string());
        }
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix('\'') {
        let end = rest.find('\'')?;
        let after = rest.get(end + 1..)?.trim_start();
        if after.starts_with(':') {
            return Some(rest[..end].to_string());
        }
        return None;
    }
    let end = trimmed.find(':')?;
    let key = trimmed[..end].trim();
    if key.is_empty() || key.contains(' ') {
        return None;
    }
    Some(key.to_string())
}

fn emit_yaml_entry(key: &str, value: &Value, indent: usize) -> Result<String, RimeError> {
    let mut tmp = Mapping::new();
    tmp.insert(yaml_str(key), value.clone());
    let rendered = serde_yaml::to_string(&Value::Mapping(tmp))
        .map_err(|err| RimeError::YamlParseError(format!("序列化 YAML 失败: {err}")))?;
    let body = rendered.trim_start_matches("---").trim_start();
    let pad = " ".repeat(indent);
    let mut out = String::new();
    for line in body.lines() {
        if line.is_empty() {
            continue;
        }
        out.push_str(&pad);
        out.push_str(line);
        out.push('\n');
    }
    Ok(out)
}

fn reconstruct_preserving(existing: &str, new_root: &Mapping) -> Result<String, RimeError> {
    let Some(Value::Mapping(new_patch)) = new_root.get(yaml_str("patch")) else {
        return serialize_custom_yaml(new_root);
    };

    let lines: Vec<&str> = existing.lines().collect();
    let patch_idx = lines.iter().position(|line| {
        indent_width(line) == 0 && line.trim() == "patch:"
            || (indent_width(line) == 0 && line.trim() == "patch: {}")
    });

    let key_indent_default = 2;
    let mut out = String::new();

    let Some(patch_idx) = patch_idx else {
        let preamble = existing.trim_end();
        if !preamble.is_empty() {
            out.push_str(preamble);
            out.push('\n');
        }
        out.push_str("patch:\n");
        for (key, value) in new_patch {
            let Some(name) = mapping_key_string(key) else {
                continue;
            };
            out.push_str(&emit_yaml_entry(&name, value, key_indent_default)?);
        }
        return Ok(out);
    };

    let old_patch = parse_yaml_mapping(existing)
        .ok()
        .and_then(|root| {
            root.get(yaml_str("patch"))
                .and_then(Value::as_mapping)
                .cloned()
        })
        .unwrap_or_default();

    let preamble = lines[..patch_idx].join("\n");
    if !preamble.is_empty() {
        out.push_str(&preamble);
        if !preamble.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str("patch:\n");

    let mut key_indent = None;
    for line in lines.iter().skip(patch_idx + 1) {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if indent_width(line) == 0 {
            break;
        }
        if parse_mapping_key(line).is_some() {
            key_indent = Some(indent_width(line));
            break;
        }
    }
    let key_indent = key_indent.unwrap_or(key_indent_default);

    let mut pending = String::new();
    let mut seen = Vec::new();
    let mut index = patch_idx + 1;
    let mut postamble = String::new();

    while index < lines.len() {
        let line = lines[index];
        let indent = indent_width(line);
        let trimmed = line.trim();

        if trimmed.is_empty() {
            pending.push('\n');
            index += 1;
            continue;
        }

        if indent == 0 {
            postamble = lines[index..].join("\n");
            break;
        }

        if trimmed.starts_with('#') {
            pending.push_str(line);
            pending.push('\n');
            index += 1;
            continue;
        }

        if indent == key_indent {
            if let Some(key) = parse_mapping_key(line) {
                let mut raw = String::new();
                raw.push_str(line);
                raw.push('\n');
                index += 1;
                while index < lines.len() {
                    let next = lines[index];
                    let next_indent = indent_width(next);
                    let next_trim = next.trim();
                    if next_trim.is_empty() {
                        let mut look = index + 1;
                        while look < lines.len() && lines[look].trim().is_empty() {
                            look += 1;
                        }
                        if look < lines.len() && indent_width(lines[look]) > key_indent {
                            raw.push_str(next);
                            raw.push('\n');
                            index += 1;
                            continue;
                        }
                        break;
                    }
                    if next_indent == 0 {
                        break;
                    }
                    if next_indent == key_indent
                        && (next_trim.starts_with('#') || parse_mapping_key(next).is_some())
                    {
                        break;
                    }
                    raw.push_str(next);
                    raw.push('\n');
                    index += 1;
                }

                seen.push(key.clone());
                match new_patch.get(yaml_str(&key)) {
                    None => pending.clear(),
                    Some(new_value) => {
                        out.push_str(&pending);
                        pending.clear();
                        if old_patch.get(yaml_str(&key)) == Some(new_value) {
                            out.push_str(&raw);
                        } else {
                            out.push_str(&emit_yaml_entry(&key, new_value, key_indent)?);
                        }
                    }
                }
                continue;
            }
        }

        pending.push_str(line);
        pending.push('\n');
        index += 1;
    }

    for (key, value) in new_patch {
        let Some(name) = mapping_key_string(key) else {
            continue;
        };
        if seen.iter().any(|item| item == &name) {
            continue;
        }
        out.push_str(&emit_yaml_entry(&name, value, key_indent)?);
    }

    if !pending.trim().is_empty() {
        out.push_str(&pending);
    }
    if !postamble.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&postamble);
        if existing.ends_with('\n') && !out.ends_with('\n') {
            out.push('\n');
        }
    }

    let parsed = parse_yaml_mapping(&out)?;
    match parsed.get(yaml_str("patch")).and_then(Value::as_mapping) {
        Some(got) if patches_semantically_eq(got, new_patch) => Ok(out),
        _ => serialize_custom_yaml(new_root),
    }
}

pub(crate) fn merge_custom_yaml<F>(existing: &str, apply: F) -> Result<String, RimeError>
where
    F: FnOnce(&mut Mapping),
{
    let mut root = parse_yaml_mapping(existing)?;
    {
        let key = yaml_str("patch");
        if !matches!(root.get(&key), Some(Value::Mapping(_))) {
            root.insert(key.clone(), Value::Mapping(Mapping::new()));
        }
        let Some(Value::Mapping(patch)) = root.get_mut(&key) else {
            return Err(RimeError::YamlParseError("无法创建 patch 映射".to_string()));
        };
        apply(patch);
    }

    if existing.trim().is_empty() {
        return serialize_custom_yaml(&root);
    }

    reconstruct_preserving(existing, &root)
}

fn nested_get<'a>(mapping: &'a Mapping, path: &str) -> Option<&'a Value> {
    let mut current = mapping;
    let parts: Vec<&str> = path.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        let value = current.get(yaml_str(part))?;
        if index + 1 == parts.len() {
            return Some(value);
        }
        current = value.as_mapping()?;
    }
    None
}

fn nested_set(mapping: &mut Mapping, path: &str, value: Value) -> bool {
    let Some((head, rest)) = path.split_once('/') else {
        mapping.insert(yaml_str(path), value);
        return true;
    };
    match mapping.get_mut(yaml_str(head)) {
        Some(Value::Mapping(child)) => nested_set(child, rest, value),
        _ => false,
    }
}

fn nested_remove(mapping: &mut Mapping, path: &str) -> bool {
    let Some((head, rest)) = path.split_once('/') else {
        return mapping.remove(yaml_str(path)).is_some();
    };
    match mapping.get_mut(yaml_str(head)) {
        Some(Value::Mapping(child)) => nested_remove(child, rest),
        _ => false,
    }
}

pub(crate) fn get_patch_path<'a>(patch: &'a Mapping, path: &str) -> Option<&'a Value> {
    patch
        .get(yaml_str(path))
        .or_else(|| nested_get(patch, path))
}

pub(crate) fn set_patch_path(patch: &mut Mapping, path: &str, value: Value) {
    let slash = yaml_str(path);
    if patch.contains_key(&slash) {
        patch.insert(slash, value);
        return;
    }
    if nested_get(patch, path).is_some() && nested_set(patch, path, value.clone()) {
        return;
    }
    patch.insert(slash, value);
}

pub(crate) fn remove_patch_path(patch: &mut Mapping, path: &str) {
    patch.remove(yaml_str(path));
    let _ = nested_remove(patch, path);
}

pub(crate) fn mapping_string(mapping: &Mapping, key: &str) -> Option<String> {
    mapping.get(yaml_str(key)).and_then(value_to_string)
}

pub(crate) fn schema_list_value(schema_ids: &[String]) -> Value {
    Value::Sequence(
        schema_ids
            .iter()
            .map(|schema_id| {
                let mut item = Mapping::new();
                item.insert(yaml_str("schema"), yaml_str(schema_id));
                Value::Mapping(item)
            })
            .collect(),
    )
}

pub(crate) fn binding_value(when: &str, accept: &str, send: &str) -> Value {
    let mut item = Mapping::new();
    item.insert(yaml_str("when"), yaml_str(when));
    item.insert(yaml_str("accept"), yaml_str(accept));
    item.insert(yaml_str("send"), yaml_str(send));
    Value::Mapping(item)
}

fn binding_signature(value: &Value) -> Option<(String, String, String)> {
    let Value::Mapping(mapping) = value else {
        return None;
    };
    Some((
        mapping_string(mapping, "when")?,
        mapping_string(mapping, "accept")?,
        mapping_string(mapping, "send")?,
    ))
}

const MANAGED_BINDINGS: &[(&str, &str, &str)] = &[
    ("paging", "Up", "Page_Up"),
    ("has_menu", "Down", "Page_Down"),
    ("has_menu", "Left", "Up"),
    ("has_menu", "Right", "Down"),
    ("has_menu", "Left", "Page_Up"),
    ("has_menu", "Right", "Page_Down"),
    ("paging", "minus", "Page_Up"),
    ("has_menu", "equal", "Page_Down"),
    ("paging", "comma", "Page_Up"),
    ("has_menu", "period", "Page_Down"),
];

pub(crate) fn is_managed_binding(value: &Value) -> bool {
    let Some(signature) = binding_signature(value) else {
        return false;
    };
    MANAGED_BINDINGS.iter().any(|(when, accept, send)| {
        signature.0 == *when && signature.1 == *accept && signature.2 == *send
    })
}

pub(crate) fn merge_key_binder_bindings(patch: &mut Mapping, managed: Vec<Value>) {
    let mut kept = Vec::new();
    if let Some(Value::Sequence(existing)) = get_patch_path(patch, "key_binder/bindings") {
        kept.extend(
            existing
                .iter()
                .filter(|item| !is_managed_binding(item))
                .cloned(),
        );
    }
    kept.extend(managed);

    if kept.is_empty() {
        remove_patch_path(patch, "key_binder/bindings");
        return;
    }

    set_patch_path(patch, "key_binder/bindings", Value::Sequence(kept));
}

pub(crate) fn collect_color_scheme_names(patch: &Mapping) -> Vec<String> {
    let mut names = Vec::new();
    let mut push = |name: String| {
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    };

    for key in patch.keys() {
        let Value::String(path) = key else {
            continue;
        };
        let Some(rest) = path.strip_prefix("preset_color_schemes/") else {
            continue;
        };
        let name = rest.split('/').next().unwrap_or(rest);
        push(name.to_string());
    }

    if let Some(Value::Mapping(schemes)) = get_patch_path(patch, "preset_color_schemes") {
        for key in schemes.keys() {
            if let Value::String(name) = key {
                push(name.clone());
            }
        }
    }

    names
}

pub(crate) fn remove_color_scheme(patch: &mut Mapping, name: &str) {
    let prefix = format!("preset_color_schemes/{name}/");
    let keys: Vec<Value> = patch
        .keys()
        .filter(|key| {
            matches!(
                key,
                Value::String(path) if path == &format!("preset_color_schemes/{name}")
                    || path.starts_with(&prefix)
            )
        })
        .cloned()
        .collect();
    for key in keys {
        patch.remove(key);
    }

    if let Some(Value::Mapping(schemes)) = patch.get_mut(yaml_str("preset_color_schemes")) {
        schemes.remove(yaml_str(name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_preserves_unknown_keys_and_updates_managed_ones() {
        let existing = r#"
patch:
  "style/font_face": "Sarasa Gothic"
  "menu/page_size": 5
  "key_binder/bindings":
    - {when: always, accept: Control+p, toggle: ascii_mode}
"#;

        let rendered = merge_custom_yaml(existing, |patch| {
            set_patch_path(patch, "menu/page_size", Value::from(9));
        })
        .expect("merge yaml");

        assert!(rendered.contains("Sarasa Gothic"));
        assert!(rendered.contains("ascii_mode"));
        assert!(rendered.contains('9'));
        assert!(!rendered.contains("page_size: 5"));
    }

    #[test]
    fn refuses_to_overwrite_invalid_yaml() {
        let err = merge_custom_yaml("patch: [", |_| {}).expect_err("invalid yaml");
        assert!(err.to_string().contains("中止写入"));
    }

    #[test]
    fn merge_keeps_user_key_bindings() {
        let existing = r#"
patch:
  "key_binder/bindings":
    - {when: always, accept: Control+p, toggle: ascii_mode}
    - {when: paging, accept: Up, send: Page_Up}
"#;
        let rendered = merge_custom_yaml(existing, |patch| {
            merge_key_binder_bindings(patch, vec![binding_value("paging", "minus", "Page_Up")]);
        })
        .expect("merge bindings");

        assert!(rendered.contains("Control+p"));
        assert!(rendered.contains("minus"));
        assert!(!rendered.contains("accept: Up"));
    }

    #[test]
    fn merge_preserves_comments_on_unmanaged_keys() {
        let existing = "# keep this header\npatch:\n  # font comment\n  \"style/font_face\": \"Sarasa Gothic\"\n  \"menu/page_size\": 5\n";
        let rendered = merge_custom_yaml(existing, |patch| {
            set_patch_path(patch, "menu/page_size", Value::from(9));
        })
        .expect("merge yaml");

        assert!(rendered.contains("# keep this header"));
        assert!(rendered.contains("# font comment"));
        assert!(rendered.contains("Sarasa Gothic"));
        assert!(rendered.contains('9'));
        assert!(!rendered.contains(": 5"));
    }
}
