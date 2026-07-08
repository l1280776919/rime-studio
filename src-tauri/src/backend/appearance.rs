use crate::backend::*;
use crate::*;
use std::{fs, path::{Path, PathBuf}};

pub(crate) fn read_appearance_config(user_dir: &Path) -> AppearanceConfig {
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

pub(crate) fn render_weasel_custom(config: &AppearanceConfig) -> String {
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

pub(crate) fn write_appearance_config(
    user_dir: &Path,
    config: &AppearanceConfig,
    include_behavior: bool,
) -> Result<(), String> {
    fs::create_dir_all(user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let path = user_dir.join("weasel.custom.yaml");
    let _ = include_behavior;
    write_text_file(&path, &render_weasel_custom(config), "写入外观配置文件失败")
}

