use crate::backend::*;
use crate::*;
use std::{ffi::OsStr, fs, path::PathBuf};

pub(crate) fn list_schemas_sync() -> Result<Vec<SchemaInfo>, RimeError> {
    let user_dir = rime_user_dir()?;
    let active_schema = read_to_string(&user_dir.join("default.custom.yaml"));
    let active = parse_schema(&active_schema);
    let enabled = parse_schema_list(&active_schema);
    let mut schemas = Vec::new();

    // Find system schemas from Weasel data directory
    let mut system_dirs: Vec<PathBuf> = Vec::new();
    if let Some(deployer) = locate_deployer() {
        if let Some(parent) = deployer.parent() {
            system_dirs.push(parent.join("data"));
        }
    }
    for root in weasel_root_from_registry() {
        system_dirs.push(root.join("data"));
    }
    for parent in [
        PathBuf::from(r"C:\Program Files\Rime"),
        PathBuf::from(r"C:\Program Files (x86)\Rime"),
    ] {
        for deployer in weasel_deployers_under(&parent) {
            if let Some(p) = deployer.parent() {
                system_dirs.push(p.join("data"));
            }
        }
        system_dirs.push(parent.join("data"));
    }

    let mut seen = std::collections::HashSet::new();

    // Scan system data dirs
    for data_dir in &system_dirs {
        if !data_dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                    continue;
                };
                if !name.ends_with(".schema.yaml") {
                    continue;
                }
                let id = name.replace(".schema.yaml", "");
                if seen.contains(&id) {
                    continue;
                }
                seen.insert(id.clone());

                let contents = fs::read_to_string(&path).unwrap_or_default();
                let schema_name =
                    parse_quoted_value(&contents, "schema/name").unwrap_or_else(|| id.clone());
                let description = parse_quoted_value(&contents, "schema/description")
                    .or_else(|| parse_string_after_key(&contents, "description:"))
                    .unwrap_or_default();

                schemas.push(SchemaInfo {
                    is_system: true,
                    is_active: active.as_ref() == Some(&id),
                    is_enabled: enabled.iter().any(|schema_id| schema_id == &id),
                    id,
                    name: schema_name,
                    description,
                    path: path.display().to_string(),
                });
            }
        }
    }

    // Scan user dir for custom schemas
    if user_dir.exists() {
        if let Ok(entries) = fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                    continue;
                };
                if !name.ends_with(".schema.yaml") && !name.ends_with(".custom.yaml") {
                    continue;
                }
                // Skip weasel.custom.yaml and default.custom.yaml
                if name == "weasel.custom.yaml" || name == "default.custom.yaml" {
                    continue;
                }
                let id = name.replace(".custom.yaml", "").replace(".schema.yaml", "");
                if seen.contains(&id) {
                    // Already listed as system; mark as having user override
                    if name.ends_with(".custom.yaml") {
                        if let Some(s) = schemas.iter_mut().find(|s| s.id == id) {
                            s.is_system = false;
                        }
                    }
                    continue;
                }
                seen.insert(id.clone());

                let contents = fs::read_to_string(&path).unwrap_or_default();
                let schema_name = parse_quoted_value(&contents, "schema/name")
                    .or_else(|| parse_string_after_key(&contents, "name:"))
                    .unwrap_or_else(|| id.clone());
                let description = parse_quoted_value(&contents, "schema/description")
                    .or_else(|| parse_string_after_key(&contents, "description:"))
                    .unwrap_or_default();

                schemas.push(SchemaInfo {
                    is_system: false,
                    is_active: active.as_ref() == Some(&id),
                    is_enabled: enabled.iter().any(|schema_id| schema_id == &id),
                    id,
                    name: schema_name,
                    description,
                    path: path.display().to_string(),
                });
            }
        }
    }

    schemas.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then(a.is_system.cmp(&b.is_system))
            .then(a.name.cmp(&b.name))
    });
    Ok(schemas)
}

pub(crate) fn copy_schema_sync(schema_id: String) -> Result<String, RimeError> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;

    let safe_id = schema_id.replace(['/', '\\'], "").replace("..", "");

    // Find the source schema file
    let system_dirs = [
        locate_deployer().and_then(|d| d.parent().map(|p| p.join("data"))),
        Some(PathBuf::from(r"C:\Program Files\Rime\weasel-0.17.4\data")),
        Some(PathBuf::from(
            r"C:\Program Files (x86)\Rime\weasel-0.17.4\data",
        )),
    ];

    let mut source: Option<PathBuf> = None;
    for dir in system_dirs.iter().flatten() {
        let candidate = dir.join(format!("{safe_id}.schema.yaml"));
        if candidate.exists() {
            source = Some(candidate);
            break;
        }
    }

    // Also check user dir
    let user_candidate = user_dir.join(format!("{safe_id}.schema.yaml"));
    if user_candidate.exists() {
        source = Some(user_candidate);
    }

    let source = source.ok_or_else(|| RimeError::SchemaError("未找到源方案文件".to_string()))?;

    // Read source and create a custom copy
    let contents = fs::read_to_string(&source)
        .map_err(|err| RimeError::FileOperationError(format!("读取方案文件失败: {err}")))?;

    // Write as .custom.yaml in user dir
    let dest_name = format!("{safe_id}.custom.yaml");
    let dest = user_dir.join(&dest_name);

    if dest.exists() {
        return Err(RimeError::SchemaError(format!(
            "{dest_name} 已存在，未自动覆盖"
        )));
    }

    // Add a header comment
    let patched = format!(
        "# {} — 从系统方案复制，由 Rime Studio 管理\n# 在此文件中添加 patch 配置即可自定义方案\n\n{}",
        safe_id, contents
    );

    write_text_file(&dest, &patched, "写入方案文件失败")?;
    Ok(dest.display().to_string())
}

pub(crate) fn sanitize_schema_id(schema_id: &str) -> String {
    schema_id
        .replace(['/', '\\'], "")
        .replace("..", "")
        .trim()
        .to_string()
}

pub(crate) fn sanitize_schema_ids(schema_ids: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    schema_ids
        .into_iter()
        .map(|schema_id| sanitize_schema_id(&schema_id))
        .filter(|schema_id| !schema_id.is_empty())
        .filter(|schema_id| seen.insert(schema_id.clone()))
        .collect()
}

pub(crate) fn render_default_custom_with_schema_list(
    config: &QuickSettingsConfig,
    schema_ids: &[String],
) -> String {
    let switch_value = if config.switch_key == "shift" {
        "commit_code"
    } else {
        "noop"
    };
    let mut default_contents = vec![
        "# Managed by Rime Studio. Previous versions are kept in RimeStudio backups.".to_string(),
        "patch:".to_string(),
        "  \"schema_list\":".to_string(),
    ];

    for schema_id in schema_ids {
        default_contents.push(format!("    - {{schema: {schema_id}}}"));
    }

    default_contents.extend([
        format!("  \"menu/page_size\": {}", config.page_size),
        format!("  \"ascii_composer/switch_key/Shift_L\": {switch_value}"),
        format!("  \"ascii_composer/switch_key/Shift_R\": {switch_value}"),
    ]);

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

pub(crate) fn save_active_schema_list_sync(
    schema_ids: Vec<String>,
) -> Result<QuickSettingsConfig, RimeError> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let safe_schema_ids = sanitize_schema_ids(schema_ids);
    if safe_schema_ids.is_empty() {
        return Err(RimeError::SchemaError(
            "至少需要启用一个输入方案".to_string(),
        ));
    }
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;

    let mut config = get_quick_settings_sync()?;
    config.schema_id = safe_schema_ids[0].clone();
    write_text_file(
        &user_dir.join("default.custom.yaml"),
        &render_default_custom_with_schema_list(&config, &safe_schema_ids),
        "写入 default.custom.yaml 失败",
    )?;

    get_quick_settings_sync()
}

pub(crate) fn set_active_schema_sync(schema_id: String) -> Result<QuickSettingsConfig, RimeError> {
    let safe_id = sanitize_schema_id(&schema_id);
    if safe_id.is_empty() {
        return Err(RimeError::SchemaError("方案 ID 不能为空".to_string()));
    }

    let user_dir = rime_user_dir()?;
    let mut schema_ids = parse_schema_list(&read_to_string(&user_dir.join("default.custom.yaml")));
    schema_ids.retain(|id| id != &safe_id);
    schema_ids.insert(0, safe_id);
    save_active_schema_list_sync(schema_ids)
}

pub(crate) fn validate_schema_path(path: String) -> Result<PathBuf, RimeError> {
    let path = PathBuf::from(path);
    if !path.exists() || !path.is_file() {
        return Err(RimeError::SchemaError("方案文件不存在".to_string()));
    }

    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return Err(RimeError::SchemaError("方案文件名无效".to_string()));
    };
    if !name.ends_with(".schema.yaml") && !name.ends_with(".custom.yaml") {
        return Err(RimeError::SchemaError("只能打开 Rime 方案文件".to_string()));
    }

    Ok(path)
}

pub(crate) fn open_schema_file_sync(path: String) -> Result<(), RimeError> {
    let path = validate_schema_path(path)?;
    reveal_in_explorer(&path)
}

pub(crate) fn open_schema_dir_sync(path: String) -> Result<(), RimeError> {
    let path = validate_schema_path(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| RimeError::SchemaError("方案文件目录无效".to_string()))?;
    open_in_explorer(parent)
}

pub(crate) fn list_community_schemas_sync() -> Result<Vec<CommunitySchema>, RimeError> {
    let installed_schemas = list_schemas_sync()
        .map(|list| {
            list.into_iter()
                .map(|s| s.id)
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();

    let presets = vec![
        CommunitySchema {
            id: "rime_ice".to_string(),
            name: "雾凇拼音 (rime-ice)".to_string(),
            description: "当前最受欢迎的 Rime 全拼/双拼输入方案，拥有高频现代词库与丰富规则配置。"
                .to_string(),
            recipe: "iDvel/rime-ice:others/recipes/full".to_string(),
            author: "iDvel".to_string(),
            tags: vec![
                "全拼".to_string(),
                "双拼".to_string(),
                "热门推荐".to_string(),
                "大词库".to_string(),
            ],
            installed: installed_schemas.contains("rime_ice"),
        },
        CommunitySchema {
            id: "mint_pinyin".to_string(),
            name: "薄荷拼音 (Oh-my-rime)".to_string(),
            description: "开箱即用、轻量易定制的现代化 Rime 拼音输入方案。".to_string(),
            recipe: "Mintimate/oh-my-rime".to_string(),
            author: "Mintimate".to_string(),
            tags: vec![
                "全拼".to_string(),
                "双拼".to_string(),
                "现代化".to_string(),
                "开箱即用".to_string(),
            ],
            installed: installed_schemas.contains("mint_pinyin")
                || installed_schemas.contains("mint"),
        },
        CommunitySchema {
            id: "clover".to_string(),
            name: "四叶草拼音 (Clover)".to_string(),
            description: "基于搜狗、百度词库整理的极简全拼/双拼方案，适合日常打字。".to_string(),
            recipe: "fkxxyz/rime-cloverpinyin".to_string(),
            author: "fkxxyz".to_string(),
            tags: vec![
                "全拼".to_string(),
                "双拼".to_string(),
                "词库丰富".to_string(),
            ],
            installed: installed_schemas.contains("clover"),
        },
        CommunitySchema {
            id: "double_pinyin_flypy".to_string(),
            name: "小鹤双拼 (Flypy)".to_string(),
            description: "高效的双拼输入方案，规则严谨，韵母键位分布科学。".to_string(),
            recipe: "rime/rime-double-pinyin".to_string(),
            author: "Rime Official".to_string(),
            tags: vec!["双拼".to_string(), "小鹤".to_string(), "高效".to_string()],
            installed: installed_schemas.contains("double_pinyin_flypy"),
        },
        CommunitySchema {
            id: "double_pinyin".to_string(),
            name: "自然码双拼 (Ziranma)".to_string(),
            description: "经典自然码双拼方案，双拼爱好者常用基础方案。".to_string(),
            recipe: "rime/rime-double-pinyin".to_string(),
            author: "Rime Official".to_string(),
            tags: vec!["双拼".to_string(), "自然码".to_string(), "经典".to_string()],
            installed: installed_schemas.contains("double_pinyin"),
        },
        CommunitySchema {
            id: "wubi86".to_string(),
            name: "五笔字型 (86版)".to_string(),
            description: "经典五笔字型形码方案，支持 86 版编码体系。".to_string(),
            recipe: "rime/rime-wubi".to_string(),
            author: "Rime Official".to_string(),
            tags: vec!["形码".to_string(), "五笔".to_string()],
            installed: installed_schemas.contains("wubi86"),
        },
        CommunitySchema {
            id: "cangjie5".to_string(),
            name: "仓颉输入法 (五代)".to_string(),
            description: "官方仓颉五代中文形码输入方案，适合繁简中文高精度录入。".to_string(),
            recipe: "rime/rime-cangjie".to_string(),
            author: "Rime Official".to_string(),
            tags: vec!["形码".to_string(), "仓颉".to_string(), "精准".to_string()],
            installed: installed_schemas.contains("cangjie5"),
        },
        CommunitySchema {
            id: "luna_pinyin".to_string(),
            name: "朙月拼音 (Luna Pinyin)".to_string(),
            description: "Rime 经典官方拼音方案，支持简繁体转换与正统拼音映射。".to_string(),
            recipe: "rime/rime-luna-pinyin".to_string(),
            author: "Rime Official".to_string(),
            tags: vec!["官方".to_string(), "经典".to_string(), "简繁体".to_string()],
            installed: installed_schemas.contains("luna_pinyin"),
        },
    ];

    Ok(presets)
}
