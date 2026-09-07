use crate::backend::*;
use crate::*;
use serde_yaml::{Mapping, Value};
use std::{fs, path::Path};

pub(crate) const SYSTEM_THEME_NAMES: &[&str] = &[
    "rime_studio_blue",
    "rime_studio_ice",
    "rime_studio_night_blue",
    "rime_studio_dark",
    "rime_studio_warm",
    "rime_studio_bamboo",
];

fn yaml_u32(value: u32) -> Value {
    Value::from(i64::from(value))
}

fn scheme_color(patch: &Mapping, name: &str, key: &str, fallback: &str) -> String {
    let slash = format!("preset_color_schemes/{name}/{key}");
    normalize_color(
        get_patch_path(patch, &slash).and_then(yaml_value_to_string),
        fallback,
    )
}

fn scheme_label(patch: &Mapping, name: &str) -> String {
    get_patch_path(patch, &format!("preset_color_schemes/{name}/name"))
        .and_then(yaml_value_to_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| name.to_string())
}

fn read_scheme_from_patch(patch: &Mapping, name: &str) -> ColorScheme {
    ColorScheme {
        name: name.to_string(),
        label: scheme_label(patch, name),
        back_color: scheme_color(patch, name, "back_color", "0xFFF8F0"),
        border_color: scheme_color(patch, name, "border_color", "0xE8CFAF"),
        text_color: scheme_color(patch, name, "text_color", "0x4A2F18"),
        candidate_text_color: scheme_color(patch, name, "candidate_text_color", "0x4A2F18"),
        comment_text_color: scheme_color(patch, name, "comment_text_color", "0x8A735E"),
        hilited_text_color: scheme_color(patch, name, "hilited_text_color", "0xFFFFFF"),
        hilited_back_color: scheme_color(patch, name, "hilited_back_color", "0xD37D2F"),
        hilited_candidate_text_color: scheme_color(
            patch,
            name,
            "hilited_candidate_text_color",
            "0xFFFFFF",
        ),
        hilited_candidate_back_color: scheme_color(
            patch,
            name,
            "hilited_candidate_back_color",
            "0xD37D2F",
        ),
    }
}

fn write_scheme_to_patch(patch: &mut Mapping, scheme: &ColorScheme) {
    let prefix = format!("preset_color_schemes/{}", scheme.name);
    set_patch_path(patch, &format!("{prefix}/name"), yaml_str(&scheme.label));
    set_patch_path(patch, &format!("{prefix}/author"), yaml_str("Rime Studio"));
    set_patch_path(
        patch,
        &format!("{prefix}/back_color"),
        yaml_str(&scheme.back_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/border_color"),
        yaml_str(&scheme.border_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/text_color"),
        yaml_str(&scheme.text_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/candidate_text_color"),
        yaml_str(&scheme.candidate_text_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/comment_text_color"),
        yaml_str(&scheme.comment_text_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/hilited_text_color"),
        yaml_str(&scheme.hilited_text_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/hilited_back_color"),
        yaml_str(&scheme.hilited_back_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/hilited_candidate_text_color"),
        yaml_str(&scheme.hilited_candidate_text_color),
    );
    set_patch_path(
        patch,
        &format!("{prefix}/hilited_candidate_back_color"),
        yaml_str(&scheme.hilited_candidate_back_color),
    );
}

fn appearance_as_scheme(config: &AppearanceConfig) -> ColorScheme {
    ColorScheme {
        name: config.theme_name.clone(),
        label: config.theme_name.clone(),
        back_color: config.back_color.clone(),
        border_color: config.border_color.clone(),
        text_color: config.text_color.clone(),
        candidate_text_color: config.candidate_text_color.clone(),
        comment_text_color: config.comment_text_color.clone(),
        hilited_text_color: config.hilited_text_color.clone(),
        hilited_back_color: config.hilited_back_color.clone(),
        hilited_candidate_text_color: config.hilited_candidate_text_color.clone(),
        hilited_candidate_back_color: config.hilited_candidate_back_color.clone(),
    }
}

pub(crate) fn read_appearance_config(user_dir: &Path) -> AppearanceConfig {
    let weasel_custom = read_to_string(&user_dir.join("weasel.custom.yaml"));
    let patch = parse_yaml_mapping(&weasel_custom)
        .ok()
        .and_then(|root| {
            root.get(yaml_str("patch"))
                .and_then(Value::as_mapping)
                .cloned()
        })
        .unwrap_or_default();

    let theme_name = parse_string_after_key(&weasel_custom, "style/color_scheme")
        .or_else(|| parse_quoted_value(&weasel_custom, "name:"))
        .unwrap_or_else(|| "rime_studio_blue".to_string());
    let default_custom = read_to_string(&user_dir.join("default.custom.yaml"));
    let current = read_scheme_from_patch(&patch, &theme_name);
    let custom_schemes = collect_color_scheme_names(&patch)
        .into_iter()
        .filter(|name| !SYSTEM_THEME_NAMES.contains(&name.as_str()))
        .map(|name| read_scheme_from_patch(&patch, &name))
        .collect();

    AppearanceConfig {
        theme_name,
        font_point: parse_u32_after_key(&weasel_custom, "style/font_point").unwrap_or(11),
        label_font_point: parse_u32_after_key(&weasel_custom, "style/label_font_point")
            .unwrap_or(10),
        font_face: parse_string_after_key(&weasel_custom, "style/font_face").unwrap_or_default(),
        label_font_face: parse_string_after_key(&weasel_custom, "style/label_font_face")
            .unwrap_or_default(),
        page_size: parse_u32_after_key(&weasel_custom, "style/page_size")
            .or_else(|| parse_u32_after_key(&default_custom, "menu/page_size"))
            .unwrap_or(7),
        switch_key: parse_string_after_key(&default_custom, "ascii_composer/switch_key/Shift_L")
            .unwrap_or_else(|| "shift".to_string()),
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
        back_color: current.back_color,
        border_color: current.border_color,
        text_color: current.text_color,
        candidate_text_color: current.candidate_text_color,
        comment_text_color: current.comment_text_color,
        hilited_text_color: current.hilited_text_color,
        hilited_back_color: current.hilited_back_color,
        hilited_candidate_text_color: current.hilited_candidate_text_color,
        hilited_candidate_back_color: current.hilited_candidate_back_color,
        custom_schemes,
    }
}

pub(crate) fn apply_appearance_patch(patch: &mut Mapping, config: &AppearanceConfig) {
    set_patch_path(patch, "style/color_scheme", yaml_str(&config.theme_name));
    set_patch_path(patch, "style/font_point", yaml_u32(config.font_point));
    set_patch_path(
        patch,
        "style/label_font_point",
        yaml_u32(config.label_font_point),
    );
    if config.font_face.trim().is_empty() {
        remove_patch_path(patch, "style/font_face");
    } else {
        set_patch_path(patch, "style/font_face", yaml_str(config.font_face.trim()));
    }
    if config.label_font_face.trim().is_empty() {
        remove_patch_path(patch, "style/label_font_face");
    } else {
        set_patch_path(
            patch,
            "style/label_font_face",
            yaml_str(config.label_font_face.trim()),
        );
    }
    set_patch_path(patch, "style/horizontal", Value::from(config.horizontal));
    set_patch_path(
        patch,
        "style/inline_preedit",
        Value::from(config.inline_preedit),
    );
    set_patch_path(
        patch,
        "style/candidate_format",
        yaml_str(&config.candidate_format),
    );
    set_patch_path(patch, "style/corner_radius", yaml_u32(config.corner_radius));
    set_patch_path(patch, "style/border_height", yaml_u32(config.border_height));
    set_patch_path(patch, "style/border_width", yaml_u32(config.border_width));
    set_patch_path(patch, "style/line_spacing", yaml_u32(config.line_spacing));
    set_patch_path(patch, "style/spacing", yaml_u32(config.spacing));

    write_scheme_to_patch(patch, &appearance_as_scheme(config));
    for scheme in &config.custom_schemes {
        write_scheme_to_patch(patch, scheme);
    }

    let keep: Vec<String> = config
        .custom_schemes
        .iter()
        .map(|scheme| scheme.name.clone())
        .chain(std::iter::once(config.theme_name.clone()))
        .collect();
    for name in collect_color_scheme_names(patch) {
        if name.starts_with("rime_studio_")
            && !SYSTEM_THEME_NAMES.contains(&name.as_str())
            && !keep.contains(&name)
        {
            remove_color_scheme(patch, &name);
        }
    }
}

pub(crate) fn merge_weasel_custom(
    existing: &str,
    config: &AppearanceConfig,
) -> Result<String, RimeError> {
    merge_custom_yaml(existing, |patch| apply_appearance_patch(patch, config))
}

pub(crate) fn write_appearance_config(
    user_dir: &Path,
    config: &AppearanceConfig,
) -> Result<(), RimeError> {
    fs::create_dir_all(user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let path = user_dir.join("weasel.custom.yaml");
    let existing = read_to_string(&path);
    write_text_file(
        &path,
        &merge_weasel_custom(&existing, config)?,
        "写入外观配置文件失败",
    )
}
