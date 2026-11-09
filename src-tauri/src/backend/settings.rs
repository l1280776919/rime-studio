use crate::backend::*;
use crate::*;
use serde_yaml::Value;
use std::{ffi::OsStr, fs, path::Path};

pub(crate) fn get_appearance_config_sync() -> Result<AppearanceConfig, RimeError> {
    let user_dir = rime_user_dir()?;
    read_appearance_config_checked(&user_dir)
}

fn binding_layers(contents: &str) -> [Vec<Value>; 2] {
    ["key_binder/bindings", "key_binder/bindings/+"].map(|path| {
        yaml_lookup(contents, path)
            .as_ref()
            .map(literal_binding_items)
            .unwrap_or_default()
    })
}

fn last_binding_pair(items: &[Value], first: Value, second: Value) -> Option<usize> {
    let first = items.iter().rposition(|item| item == &first)?;
    let second = items.iter().rposition(|item| item == &second)?;
    Some(first.max(second))
}

pub(crate) fn detect_paging_keys(contents: &str) -> String {
    // Later additions override earlier keys of the same condition in Rime.
    // Only infer a Studio setting from a complete pair with matching conditions.
    for items in binding_layers(contents).iter().rev() {
        let mode = [
            ("Up", "Down", "arrow_keys"),
            ("minus", "equal", "minus_equal"),
            ("comma", "period", "comma_period"),
        ]
        .into_iter()
        .filter_map(|(previous, next, mode)| {
            last_binding_pair(
                items,
                binding_value("paging", previous, "Page_Up"),
                binding_value("has_menu", next, "Page_Down"),
            )
            .map(|position| (position, mode))
        })
        .max_by_key(|(position, _)| *position);
        if let Some((_, mode)) = mode {
            return mode.to_string();
        }
    }
    "comma_period".to_string()
}

pub(crate) fn detect_navigation_keys(contents: &str) -> String {
    for items in binding_layers(contents).iter().rev() {
        for (previous, next) in [("Up", "Down"), ("Page_Up", "Page_Down")] {
            if last_binding_pair(
                items,
                binding_value("has_menu", "Left", previous),
                binding_value("has_menu", "Right", next),
            )
            .is_some()
            {
                return "left_right".to_string();
            }
        }
    }
    "up_down".to_string()
}

pub(crate) fn get_quick_settings_sync() -> Result<QuickSettingsConfig, RimeError> {
    let user_dir = rime_user_dir()?;
    let default_custom = read_user_custom_config(&user_dir, "default.custom.yaml")?;
    let appearance = read_appearance_config_checked(&user_dir)?;
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
        schema_id: parse_schema(&default_custom).unwrap_or_default(),
        schema_list: parse_schema_list(&default_custom),
        page_size: parse_u32_after_key(&default_custom, "menu/page_size")
            .unwrap_or(appearance.page_size),
        switch_key: switch_key.to_string(),
        paging_keys,
        navigation_keys,
        horizontal: appearance.horizontal,
        inline_preedit: appearance.inline_preedit,
    })
}

pub(crate) fn build_text_diff(old_contents: &str, new_contents: &str) -> Vec<String> {
    if old_contents == new_contents {
        return Vec::new();
    }

    let diff = similar::TextDiff::from_lines(old_contents, new_contents);
    let mut result = Vec::new();

    for change in diff.iter_all_changes() {
        let line = change.value().trim_end_matches(['\r', '\n']);
        match change.tag() {
            similar::ChangeTag::Delete => {
                result.push(format!("- {line}"));
            }
            similar::ChangeTag::Insert => {
                result.push(format!("+ {line}"));
            }
            similar::ChangeTag::Equal => {}
        }
    }

    result
}

pub(crate) fn preview_file(user_dir: &Path, name: &str, new_contents: String) -> ConfigPreviewFile {
    let path = user_dir.join(name);
    let old_contents = read_to_string(&path);
    let diff_lines = build_text_diff(&old_contents, &new_contents);

    ConfigPreviewFile {
        name: name.to_string(),
        path: path.display().to_string(),
        changed: !diff_lines.is_empty(),
        diff_lines,
    }
}

fn merge_weasel_quick_settings(
    existing: &str,
    config: &QuickSettingsConfig,
) -> Result<String, RimeError> {
    // Quick settings own only these three keys; changing paging must not rewrite themes or geometry.
    merge_custom_yaml(existing, |patch| {
        set_patch_path(patch, "style/page_size", Value::from(config.page_size));
        set_patch_path(patch, "style/horizontal", Value::from(config.horizontal));
        set_patch_path(
            patch,
            "style/inline_preedit",
            Value::from(config.inline_preedit),
        );
    })
}

pub(crate) fn preview_quick_settings_sync(
    config: QuickSettingsConfig,
) -> Result<ConfigPreview, RimeError> {
    let user_dir = rime_user_dir()?;

    let default_existing = read_user_custom_config(&user_dir, "default.custom.yaml")?;
    let weasel_existing = read_user_custom_config(&user_dir, "weasel.custom.yaml")?;

    Ok(ConfigPreview {
        files: vec![
            preview_file(
                &user_dir,
                "default.custom.yaml",
                merge_default_custom(
                    &default_existing,
                    &config,
                    &promoted_schema_ids(&default_existing, &config.schema_id)?,
                )?,
            ),
            preview_file(
                &user_dir,
                "weasel.custom.yaml",
                merge_weasel_quick_settings(&weasel_existing, &config)?,
            ),
        ],
    })
}

pub(crate) fn save_quick_settings_sync(
    config: QuickSettingsConfig,
) -> Result<QuickSettingsConfig, RimeError> {
    let _config_guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::SettingsError(format!("创建 Rime 目录失败: {err}")))?;
    let default_path = resolve_user_relative_path(&user_dir, "default.custom.yaml", false)?;
    let weasel_path = resolve_user_relative_path(&user_dir, "weasel.custom.yaml", false)?;
    let existing_default = read_optional_config(&default_path)?;
    let existing_weasel = read_optional_config(&weasel_path)?;
    let schemas = promoted_schema_ids(&existing_default, &config.schema_id)?;
    // Validate and render both documents before changing either one.
    let default_contents = merge_default_custom(&existing_default, &config, &schemas)?;
    let weasel_contents = merge_weasel_quick_settings(&existing_weasel, &config)?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(
        &default_path,
        &default_contents,
        "写入 default.custom.yaml 失败",
    )?;
    write_text_file(
        &weasel_path,
        &weasel_contents,
        "写入 weasel.custom.yaml 失败",
    )?;
    get_quick_settings_sync()
}

pub(crate) fn push_check(
    checks: &mut Vec<ConfigHealthCheck>,
    name: &str,
    status: &str,
    detail: String,
) {
    checks.push(ConfigHealthCheck {
        name: name.to_string(),
        status: status.to_string(),
        detail,
    });
}

pub(crate) fn patch_preamble_is_clean(contents: &str) -> bool {
    contents
        .lines()
        .take_while(|line| line.trim() != "patch:")
        .all(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
}

pub(crate) fn count_patch_key(contents: &str, key: &str) -> usize {
    contents
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start().trim_matches('"');
            trimmed.starts_with(key)
        })
        .count()
}

pub(crate) fn first_patch_string(contents: &str, key: &str) -> Option<String> {
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

pub(crate) fn first_plain_value(contents: &str, key: &str) -> Option<String> {
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

pub(crate) fn nested_plain_value(contents: &str, section: &str, key: &str) -> Option<String> {
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

pub(crate) fn inspect_config_health_sync() -> Result<ConfigHealthReport, RimeError> {
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
        if has_lmdg_grammar_patch(&rime_ice_custom) {
            push_check(
                &mut checks,
                "万象语法模型",
                "ok",
                "已检测到 LMDG 语法模型 patch".to_string(),
            );
        }
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

fn ensure_repair_deployed() -> Result<(), RimeError> {
    let result = deploy_rime_internal(None)?;
    if !result.success {
        return Err(RimeError::SettingsError(format!(
            "配置已保存，但修复部署失败: {}",
            result.message
        )));
    }
    Ok(())
}

pub(crate) fn repair_config_health_sync() -> Result<ConfigHealthReport, RimeError> {
    let quick = get_quick_settings_sync()?;
    let appearance = get_appearance_config_sync()?;

    save_quick_settings_sync(quick)?;
    save_appearance_config_sync(appearance)?;
    ensure_repair_deployed()?;

    inspect_config_health_sync()
}

pub(crate) fn repair_config_health_item_sync(
    name: String,
) -> Result<ConfigHealthReport, RimeError> {
    let name = name.trim();
    match name {
        "default.custom.yaml" | "方案列表" | "候选数量合并" => {
            let quick = get_quick_settings_sync()?;
            save_quick_settings_sync(quick)?;
            if name == "候选数量合并" {
                ensure_repair_deployed()?;
            }
        }
        "weasel.custom.yaml" | "主题配置" | "主题合并" => {
            let appearance = get_appearance_config_sync()?;
            save_appearance_config_sync(appearance)?;
            if name == "主题合并" {
                ensure_repair_deployed()?;
            }
        }
        "雾凇组件配置" | "繁体预设" => {
            let settings = get_rime_ice_settings_sync()?;
            save_rime_ice_settings_sync(settings)?;
        }
        _ => {
            return Err(RimeError::SettingsError(format!(
                "暂不支持单项修复: {name}"
            )));
        }
    }

    inspect_config_health_sync()
}

pub(crate) fn parse_patch_bool(contents: &str, key: &str, fallback: bool) -> bool {
    first_patch_string(contents, key)
        .and_then(|value| value.parse::<u32>().ok())
        .map(|value| value != 0)
        .unwrap_or(fallback)
}

pub(crate) fn get_rime_ice_settings_sync() -> Result<RimeIceSettings, RimeError> {
    let user_dir = rime_user_dir()?;
    let custom = read_user_custom_config(&user_dir, "rime_ice.custom.yaml")?;
    let detected_pairs = detect_fuzzy_pinyin_pairs(&custom);
    let has_fuzzy = !detected_pairs.is_empty();
    Ok(RimeIceSettings {
        ascii_punct: parse_patch_bool(&custom, "switches/@1/reset", false),
        traditionalization: parse_patch_bool(&custom, "switches/@2/reset", false),
        emoji: parse_patch_bool(&custom, "switches/@3/reset", true),
        full_shape: parse_patch_bool(&custom, "switches/@4/reset", false),
        search_single_char: parse_patch_bool(&custom, "switches/@5/reset", false),
        fuzzy_pinyin: has_fuzzy,
        fuzzy_pairs: if has_fuzzy {
            Some(detected_pairs)
        } else {
            None
        },
        traditional_preset: first_patch_string(&custom, "traditionalize/opencc_config")
            .unwrap_or_else(|| "s2t.json".to_string()),
    })
}

pub(crate) fn has_lmdg_grammar_patch(contents: &str) -> bool {
    contents.contains("wanxiang-lts-zh-hans")
        || contents.contains("translator/contextual_suggestions")
}

#[derive(Clone, Copy)]
pub(crate) enum LmdgPatchAction {
    Keep,
    Enable,
    Disable,
}

fn apply_lmdg_grammar_patch(patch: &mut serde_yaml::Mapping, enable: bool) {
    if enable {
        let mut grammar = serde_yaml::Mapping::new();
        grammar.insert(yaml_str("language"), yaml_str("wanxiang-lts-zh-hans"));
        grammar.insert(yaml_str("collocation_max_length"), Value::from(5));
        grammar.insert(yaml_str("collocation_min_length"), Value::from(2));
        set_patch_path(patch, "grammar", Value::Mapping(grammar));
        set_patch_path(
            patch,
            "translator/contextual_suggestions",
            Value::from(true),
        );
        set_patch_path(patch, "translator/max_homophones", Value::from(7));
        set_patch_path(patch, "translator/max_homographs", Value::from(7));
        return;
    }

    remove_patch_path(patch, "grammar");
    remove_patch_path(patch, "translator/contextual_suggestions");
    remove_patch_path(patch, "translator/max_homophones");
    remove_patch_path(patch, "translator/max_homographs");
}

pub(crate) fn apply_rime_ice_patch(
    patch: &mut serde_yaml::Mapping,
    settings: &RimeIceSettings,
    lmdg: LmdgPatchAction,
) -> Result<(), RimeError> {
    let reset = |value: bool| Value::from(if value { 1 } else { 0 });
    set_patch_path(patch, "switches/@1/reset", reset(settings.ascii_punct));
    set_patch_path(
        patch,
        "switches/@2/reset",
        reset(settings.traditionalization),
    );
    set_patch_path(patch, "switches/@3/reset", reset(settings.emoji));
    set_patch_path(patch, "switches/@4/reset", reset(settings.full_shape));
    set_patch_path(
        patch,
        "switches/@5/reset",
        reset(settings.search_single_char),
    );
    set_patch_path(
        patch,
        "traditionalize/opencc_config",
        yaml_str(&settings.traditional_preset),
    );

    merge_fuzzy_pinyin_rules(patch, settings)?;

    match lmdg {
        LmdgPatchAction::Keep => {}
        LmdgPatchAction::Enable => apply_lmdg_grammar_patch(patch, true),
        LmdgPatchAction::Disable => apply_lmdg_grammar_patch(patch, false),
    }
    Ok(())
}

pub(crate) fn merge_rime_ice_custom(
    existing: &str,
    settings: &RimeIceSettings,
    lmdg: LmdgPatchAction,
) -> Result<String, RimeError> {
    try_merge_custom_yaml(existing, |patch| {
        apply_rime_ice_patch(patch, settings, lmdg)
    })
}

pub(crate) fn save_rime_ice_settings_sync(
    settings: RimeIceSettings,
) -> Result<RimeIceSettings, RimeError> {
    let _config_guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::SettingsError(format!("创建 Rime 目录失败: {err}")))?;
    let custom_path = resolve_user_relative_path(&user_dir, "rime_ice.custom.yaml", false)?;
    let custom = read_optional_config(&custom_path)?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(
        &custom_path,
        &merge_rime_ice_custom(&custom, &settings, LmdgPatchAction::Keep)?,
        "写入 rime_ice.custom.yaml 失败",
    )?;
    Ok(settings)
}

pub(crate) fn preview_appearance_config_sync(
    config: AppearanceConfig,
) -> Result<ConfigPreview, RimeError> {
    let user_dir = rime_user_dir()?;
    let existing = read_user_custom_config(&user_dir, "weasel.custom.yaml")?;
    Ok(ConfigPreview {
        files: vec![preview_file(
            &user_dir,
            "weasel.custom.yaml",
            merge_weasel_custom(&existing, &config)?,
        )],
    })
}

pub(crate) fn preview_rime_ice_settings_sync(
    settings: RimeIceSettings,
) -> Result<ConfigPreview, RimeError> {
    let user_dir = rime_user_dir()?;
    let existing = read_user_custom_config(&user_dir, "rime_ice.custom.yaml")?;
    Ok(ConfigPreview {
        files: vec![preview_file(
            &user_dir,
            "rime_ice.custom.yaml",
            merge_rime_ice_custom(&existing, &settings, LmdgPatchAction::Keep)?,
        )],
    })
}

pub(crate) fn save_appearance_config_sync(
    config: AppearanceConfig,
) -> Result<AppearanceConfig, RimeError> {
    let _config_guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::SettingsError(format!("创建 Rime 目录失败: {err}")))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_appearance_config(&user_dir, &config)?;

    read_appearance_config_checked(&user_dir)
}

pub(crate) fn list_backups_sync() -> Result<Vec<BackupEntry>, RimeError> {
    let user_dir = rime_user_dir()?;
    list_backup_dirs(&user_dir)
}

pub(crate) fn create_backup_with_note_sync(note: Option<String>) -> Result<BackupEntry, RimeError> {
    let _config_guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::SettingsError(format!("创建 Rime 目录失败: {err}")))?;
    let backup_dir = backup_user_config_with_note(&user_dir, BackupKind::Manual, note.as_deref())?;
    let backup_name = backup_dir
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("backup-rime-studio")
        .to_string();

    list_backup_dirs(&user_dir)?
        .into_iter()
        .find(|backup| backup.name == backup_name)
        .ok_or_else(|| RimeError::BackupError("备份已创建但无法列出".to_string()))
}

pub(crate) fn open_rime_user_dir_sync() -> Result<(), RimeError> {
    open_in_explorer(&rime_user_dir()?)
}

pub(crate) fn open_config_file_sync(name: String) -> Result<(), RimeError> {
    validate_config_relpath(&name)?;
    let path = join_user_rel(&rime_user_dir()?, &name);
    if !path.exists() || !path.is_file() {
        return Err(RimeError::ConfigNotFound("配置文件不存在".to_string()));
    }

    reveal_in_explorer(&path)
}

pub(crate) fn open_sync_dir_sync() -> Result<(), RimeError> {
    let path = rime_user_dir()?.join("sync");
    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|err| RimeError::FileOperationError(format!("创建同步目录失败: {err}")))?;
    }
    open_in_explorer(&path)
}

pub(crate) fn preview_backup_sync(backup_name: String) -> Result<ConfigPreview, RimeError> {
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    let mut files = Vec::new();
    for relative in collect_snapshot_files(&backup_dir)? {
        let source = backup_dir.join(&relative);
        let name = relative.to_string_lossy().replace('\\', "/");
        let new_contents = fs::read_to_string(&source).map_err(|err| {
            RimeError::BackupError(format!("读取备份 {} 失败: {err}", relative.display()))
        })?;
        files.push(preview_file(&user_dir, &name, new_contents));
    }
    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(ConfigPreview { files })
}

pub(crate) fn open_plum_dir_sync() -> Result<(), RimeError> {
    open_in_explorer(&app_data_dir()?.join("plum"))
}

pub(crate) fn open_backup_dir_sync(backup_name: String) -> Result<(), RimeError> {
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    open_in_explorer(&backup_dir)
}

pub(crate) fn restore_backup_sync(backup_name: String) -> Result<RestoreResult, RimeError> {
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    restore_backup_dir(&user_dir, &backup_dir)
}

pub(crate) fn delete_backup_sync(backup_name: String) -> Result<(), RimeError> {
    let _backup_guard = lock_backup_operation()?;
    let user_dir = rime_user_dir()?;
    let backup_dir = validated_backup_dir(&user_dir, &backup_name)?;
    fs::remove_dir_all(&backup_dir)
        .map_err(|err| RimeError::BackupError(format!("删除备份失败: {err}")))
}

pub(crate) fn delete_dictionary_sync(dict_name: String) -> Result<(), RimeError> {
    let _config_guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;
    fs::remove_file(&path)
        .map_err(|err| RimeError::FileOperationError(format!("删除词库失败: {err}")))
}
