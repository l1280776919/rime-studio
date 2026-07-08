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

fn render_quick_default_custom(config: &QuickSettingsConfig) -> String {
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

fn build_text_diff(old_contents: &str, new_contents: &str) -> Vec<String> {
    if old_contents == new_contents {
        return Vec::new();
    }

    let old_lines = old_contents.lines().collect::<Vec<_>>();
    let new_lines = new_contents.lines().collect::<Vec<_>>();
    let max_len = old_lines.len().max(new_lines.len());
    let mut diff = Vec::new();

    for index in 0..max_len {
        match (old_lines.get(index), new_lines.get(index)) {
            (Some(old), Some(new)) if old == new => {}
            (Some(old), Some(new)) => {
                diff.push(format!("- {}", old));
                diff.push(format!("+ {}", new));
            }
            (Some(old), None) => diff.push(format!("- {}", old)),
            (None, Some(new)) => diff.push(format!("+ {}", new)),
            (None, None) => {}
        }
    }

    diff
}

fn preview_file(user_dir: &Path, name: &str, new_contents: String) -> ConfigPreviewFile {
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

fn preview_quick_settings_sync(config: QuickSettingsConfig) -> Result<ConfigPreview, String> {
    let user_dir = rime_user_dir()?;
    let mut appearance = read_appearance_config(&user_dir);
    appearance.page_size = config.page_size;
    appearance.switch_key = config.switch_key.clone();
    appearance.horizontal = config.horizontal;
    appearance.inline_preedit = config.inline_preedit;

    Ok(ConfigPreview {
        files: vec![
            preview_file(
                &user_dir,
                "default.custom.yaml",
                render_quick_default_custom(&config),
            ),
            preview_file(
                &user_dir,
                "weasel.custom.yaml",
                render_weasel_custom(&appearance),
            ),
        ],
    })
}

fn save_quick_settings_sync(config: QuickSettingsConfig) -> Result<QuickSettingsConfig, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    let default_custom_path = user_dir.join("default.custom.yaml");
    write_text_file(
        &default_custom_path,
        &render_quick_default_custom(&config),
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

fn repair_config_health_item_sync(name: String) -> Result<ConfigHealthReport, String> {
    let name = name.trim();
    match name {
        "default.custom.yaml" | "方案列表" | "候选数量合并" => {
            let quick = get_quick_settings_sync().unwrap_or(QuickSettingsConfig {
                schema_id: "luna_pinyin_simp".to_string(),
                page_size: 5,
                switch_key: "shift".to_string(),
                paging_keys: "comma_period".to_string(),
                navigation_keys: "up_down".to_string(),
                horizontal: true,
                inline_preedit: true,
            });
            save_quick_settings_sync(quick)?;
            if name == "候选数量合并" {
                let _ = deploy_rime_internal();
            }
        }
        "weasel.custom.yaml" | "主题配置" | "主题合并" => {
            let appearance = get_appearance_config_sync()?;
            save_appearance_config_sync(appearance)?;
            if name == "主题合并" {
                let _ = deploy_rime_internal();
            }
        }
        "雾凇组件配置" | "繁体预设" => {
            let settings = get_rime_ice_settings_sync()?;
            save_rime_ice_settings_sync(settings)?;
        }
        _ => {
            return Err(format!("暂不支持单项修复: {name}"));
        }
    }

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
        fuzzy_pinyin: has_fuzzy_pinyin_patch(&custom),
        traditional_preset: first_patch_string(&custom, "traditionalize/opencc_config")
            .unwrap_or_else(|| "s2t.json".to_string()),
    })
}

fn has_lmdg_grammar_patch(contents: &str) -> bool {
    contents.contains("wanxiang-lts-zh-hans")
        || contents.contains("translator/contextual_suggestions")
}

fn has_fuzzy_pinyin_patch(contents: &str) -> bool {
    contents.contains("speller/algebra/+")
        || contents.contains("derive/^([zcs])h/$1/")
        || contents.contains("derive/ang$/an/")
}

fn fuzzy_pinyin_algebra_rules() -> Vec<&'static str> {
    vec![
        "derive/^([zcs])h/$1/",
        "derive/^([zcs])([^h])/$1h$2/",
        "derive/^l/n/",
        "derive/^n/l/",
        "derive/^f/h/",
        "derive/^h/f/",
        "derive/^l/r/",
        "derive/^r/l/",
        "derive/^g/k/",
        "derive/^k/g/",
        "derive/ang$/an/",
        "derive/an$/ang/",
        "derive/eng$/en/",
        "derive/en$/eng/",
        "derive/in$/ing/",
        "derive/ing$/in/",
        "derive/ian$/iang/",
        "derive/iang$/ian/",
        "derive/uan$/uang/",
        "derive/uang$/uan/",
        "derive/ai$/an/",
        "derive/an$/ai/",
        "derive/ong$/un/",
        "derive/un$/ong/",
        "derive/ong$/on/",
        "derive/iong$/un/",
        "derive/un$/iong/",
        "derive/ong$/eng/",
        "derive/eng$/ong/",
        "derive/^fei$/hui/",
        "derive/^hui$/fei/",
        "derive/^hu$/fu/",
        "derive/^fu$/hu/",
        "derive/^wang$/huang/",
        "derive/^huang$/wang/",
        "derive/un$/uen/",
        "derive/ui$/uei/",
        "derive/iu$/iou/",
    ]
}

fn render_rime_ice_custom(
    settings: &RimeIceSettings,
    enable_lmdg_grammar: bool,
    enable_fuzzy_pinyin: bool,
) -> String {
    let bool_to_reset = |value: bool| if value { 1 } else { 0 };
    let mut lines = vec![
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
    ];

    if enable_lmdg_grammar {
        lines.extend([
            "  grammar:".to_string(),
            "    language: wanxiang-lts-zh-hans".to_string(),
            "    collocation_max_length: 5".to_string(),
            "    collocation_min_length: 2".to_string(),
            "  translator/contextual_suggestions: true".to_string(),
            "  translator/max_homophones: 7".to_string(),
            "  translator/max_homographs: 7".to_string(),
        ]);
    }

    if enable_fuzzy_pinyin {
        lines.push("  \"speller/algebra/+\":".to_string());
        for rule in fuzzy_pinyin_algebra_rules() {
            lines.push(format!("    - {rule}"));
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

fn save_rime_ice_settings_sync(settings: RimeIceSettings) -> Result<RimeIceSettings, String> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir).map_err(|err| format!("创建 Rime 目录失败: {err}"))?;
    let custom_path = user_dir.join("rime_ice.custom.yaml");
    let custom = read_to_string(&custom_path);
    let keep_lmdg_grammar = has_lmdg_grammar_patch(&custom);
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(
        &custom_path,
        &render_rime_ice_custom(&settings, keep_lmdg_grammar, settings.fuzzy_pinyin),
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

fn download_github_release_installer<F>(
    api_url: &str,
    asset_filter: F,
    missing_asset_message: &str,
    download_message_prefix: &str,
) -> Result<RimeDownloadResult, String>
where
    F: Fn(&str) -> bool,
{
    let response = ureq::get(api_url)
        .set("User-Agent", "RimeStudio/0.1")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|err| format!("获取发布信息失败: {err}"))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|err| format!("解析发布信息失败: {err}"))?;

    let assets = json["assets"].as_array().ok_or("未找到发布资源")?;
    let installer = assets
        .iter()
        .filter_map(|asset| {
            let name = asset["name"].as_str().unwrap_or("");
            if asset_filter(name) {
                Some((
                    name.to_string(),
                    asset["browser_download_url"].as_str()?.to_string(),
                ))
            } else {
                None
            }
        })
        .max_by_key(|(name, _)| name.contains("install"))
        .ok_or(missing_asset_message)?;

    let download_url = installer.1;
    let filename = installer.0;

    let dest_dir = app_data_dir()?;
    fs::create_dir_all(&dest_dir).map_err(|err| format!("创建下载目录失败: {err}"))?;
    let dest_path = dest_dir.join(&filename);

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
        message: format!("{download_message_prefix} {filename}"),
    })
}

fn download_rime_installer_sync() -> Result<RimeDownloadResult, String> {
    download_github_release_installer(
        "https://api.github.com/repos/rime/weasel/releases/latest",
        |name| name.ends_with(".exe"),
        "未找到合适的小狼毫安装包",
        "已下载",
    )
}

fn download_git_installer_sync() -> Result<RimeDownloadResult, String> {
    download_github_release_installer(
        "https://api.github.com/repos/git-for-windows/git/releases/latest",
        |name| name.starts_with("Git-") && name.ends_with(".exe") && name.contains("64-bit"),
        "未找到合适的 Git for Windows 安装包",
        "已下载 Git 安装包",
    )
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

