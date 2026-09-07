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
    serialize_custom_yaml(&root)
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
}
