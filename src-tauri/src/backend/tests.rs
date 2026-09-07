#[cfg(test)]
mod tests {
    use crate::backend::*;
    use crate::*;
    use std::{
        env,
        ffi::OsStr,
        fs,
        path::PathBuf,
        process::{self},
    };

    #[test]
    fn parses_schema_list_from_yaml_patch() {
        let contents = r#"
patch:
  schema_list:
    - schema: rime_ice
    - schema: luna_pinyin
"#;

        assert_eq!(
            parse_schema_list(contents),
            vec!["rime_ice".to_string(), "luna_pinyin".to_string()]
        );
        assert_eq!(parse_schema(contents), Some("rime_ice".to_string()));
    }

    #[test]
    fn parses_patch_values_from_yaml() {
        let contents = r#"
patch:
  "menu/page_size": 9
  "style/horizontal": false
  "style/color_scheme": rime_studio_blue
"#;

        assert_eq!(parse_u32_after_key(contents, "menu/page_size"), Some(9));
        assert_eq!(
            parse_bool_after_key(contents, "style/horizontal"),
            Some(false)
        );
        assert_eq!(
            parse_string_after_key(contents, "style/color_scheme"),
            Some("rime_studio_blue".to_string())
        );
    }

    #[test]
    fn limits_managed_backup_file_names() {
        assert!(is_managed_config_file("default.custom.yaml"));
        assert!(is_managed_config_file("sogou_ext.dict.yaml"));
        assert!(is_managed_config_file("custom_phrase.txt"));
        assert!(!is_managed_config_file("installer.exe"));
        assert!(!is_managed_config_file("notes.txt"));
    }

    #[test]
    fn reads_backup_kind_from_new_and_legacy_names() {
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-manual-123"),
            "manual"
        );
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-before-save-123"),
            "before-save"
        );
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-before-restore-123"),
            "before-restore"
        );
        assert_eq!(
            backup_kind_from_name("backup-rime-studio-before-install-123"),
            "before-install"
        );
        assert_eq!(backup_kind_from_name("backup-rime-studio-123"), "manual");
    }

    #[test]
    fn creates_unique_backup_dir_names() {
        let dir = env::temp_dir().join(format!(
            "rime-studio-backup-kind-test-{}-{}",
            process::id(),
            timestamp()
        ));
        fs::create_dir_all(&dir).expect("create test dir");

        let first =
            create_unique_backup_dir(&dir, BackupKind::BeforeSave).expect("first backup dir");
        let second =
            create_unique_backup_dir(&dir, BackupKind::BeforeSave).expect("second backup dir");

        assert_ne!(first, second);
        assert!(first
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .starts_with("backup-rime-studio-before-save-"));
        assert!(second
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .starts_with("backup-rime-studio-before-save-"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn prunes_only_old_auto_backups() {
        let dir = env::temp_dir().join(format!(
            "rime-studio-auto-backup-prune-test-{}-{}",
            process::id(),
            timestamp()
        ));
        fs::create_dir_all(&dir).expect("create test dir");

        for name in [
            "backup-rime-studio-before-save-100",
            "backup-rime-studio-before-save-101",
            "backup-rime-studio-before-install-102",
            "backup-rime-studio-before-restore-103",
            "backup-rime-studio-manual-104",
            "backup-rime-studio-legacy",
        ] {
            fs::create_dir_all(dir.join(name)).expect("create backup dir");
        }

        let removed = prune_old_auto_backups(&dir, 2).expect("prune auto backups");
        let remaining = fs::read_dir(&dir)
            .expect("read test dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect::<Vec<_>>();
        let auto_remaining = remaining
            .iter()
            .filter(|name| is_auto_backup_kind(&backup_kind_from_name(name)))
            .count();

        assert_eq!(removed, 2);
        assert_eq!(auto_remaining, 2);
        assert!(remaining.contains(&"backup-rime-studio-manual-104".to_string()));
        assert!(remaining.contains(&"backup-rime-studio-legacy".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn builds_line_diff_for_changed_text() {
        let diff = build_text_diff("a\nb\nc\n", "a\nx\nc\nd\n");

        assert_eq!(
            diff,
            vec!["- b".to_string(), "+ x".to_string(), "+ d".to_string()]
        );
        assert!(build_text_diff("same\n", "same\n").is_empty());
    }

    #[test]
    fn write_text_file_replaces_existing_contents() {
        let dir = env::temp_dir().join(format!(
            "rime-studio-test-{}-{}",
            process::id(),
            timestamp()
        ));
        let path = dir.join("default.custom.yaml");

        write_text_file(&path, "old", "测试写入").expect("initial write");
        write_text_file(&path, "new", "测试写入").expect("replacement write");

        assert_eq!(fs::read_to_string(&path).expect("read result"), "new");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn parses_tab_separated_dictionary_entries() {
        let contents = "深度学习\tshen du xue xi\t10\n# comment\n空行\tkong hang\n";
        let (entries, skipped) = parse_text_dictionary_entries(contents);

        assert_eq!(skipped, 0);
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0],
            ("深度学习".to_string(), "shen du xue xi".to_string(), 10)
        );
        assert_eq!(entries[1], ("空行".to_string(), "kong hang".to_string(), 1));
    }

    #[test]
    fn removes_duplicate_dictionary_entry_lines_only() {
        let contents = [
            "# Imported by Rime Studio.",
            "---",
            "name: custom",
            "sort: by_weight",
            "...",
            "# duplicated comments should stay",
            "# duplicated comments should stay",
            "深度学习\tshen du xue xi\t10",
            "深度学习\tshen du xue xi\t10",
            "机器学习\tji qi xue xi\t8",
            "",
        ]
        .join("\n");

        let (cleaned, removed) = remove_duplicate_dictionary_lines(&contents);

        assert_eq!(removed, 1);
        assert_eq!(
            cleaned.matches("# duplicated comments should stay").count(),
            2
        );
        assert_eq!(cleaned.matches("深度学习\tshen du xue xi\t10").count(), 1);
        assert!(cleaned.contains("机器学习\tji qi xue xi\t8"));
    }

    #[test]
    fn sanitizes_dictionary_file_names() {
        assert_eq!(sanitize_dict_file_name("sogou.scel"), "sogou.dict.yaml");
        assert_eq!(
            sanitize_dict_file_name("custom.dict.yaml"),
            "custom.dict.yaml"
        );
        assert_eq!(
            sanitize_dict_file_name("../bad name.txt"),
            "bad_name.dict.yaml"
        );
    }

    #[test]
    fn parses_scel_after_dynamic_pinyin_table() {
        let mut data = vec![0u8; 0x1540];
        data.extend(2u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(2u16.to_le_bytes());
        data.extend("a".encode_utf16().flat_map(u16::to_le_bytes));
        data.extend(1u16.to_le_bytes());
        data.extend(4u16.to_le_bytes());
        data.extend("an".encode_utf16().flat_map(u16::to_le_bytes));

        data.extend(1u16.to_le_bytes());
        data.extend(4u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(1u16.to_le_bytes());
        data.extend(4u16.to_le_bytes());
        data.extend("阿安".encode_utf16().flat_map(u16::to_le_bytes));
        data.extend(0u16.to_le_bytes());

        let (entries, skipped) = parse_scel_entries(&data).expect("parse scel");
        assert_eq!(skipped, 0);
        assert_eq!(entries, vec![("阿安".to_string(), "a an".to_string(), 1)]);
    }

    #[test]
    fn parses_sogou_bin_record() {
        let mut data = b"SGPU".to_vec();
        data.resize(32, 0);
        data.extend([0, 1, 0, 3]);
        data.extend(6u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(173u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(16u16.to_le_bytes());
        data.extend(6u16.to_le_bytes());
        data.extend("阿里啊".encode_utf16().flat_map(u16::to_le_bytes));
        data.extend(6u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());
        data.extend(173u16.to_le_bytes());
        data.extend(0u16.to_le_bytes());

        let (entries, skipped) = parse_sogou_bin_entries(&data).expect("parse sogou bin");
        assert_eq!(skipped, 0);
        assert_eq!(
            entries,
            vec![("阿里啊".to_string(), "a li a".to_string(), 1)]
        );
    }

    #[test]
    fn fuzzy_pinyin_rules_do_not_mix_g_and_k() {
        let rules = fuzzy_pinyin_algebra_rules();

        assert!(!rules.contains(&"derive/^g/k/"));
        assert!(!rules.contains(&"derive/^k/g/"));
    }

    fn sample_appearance() -> AppearanceConfig {
        AppearanceConfig {
            theme_name: "rime_studio_blue".to_string(),
            font_point: 12,
            label_font_point: 10,
            font_face: "Sarasa Gothic".to_string(),
            label_font_face: String::new(),
            page_size: 7,
            switch_key: "shift".to_string(),
            horizontal: true,
            inline_preedit: true,
            candidate_format: "%c. %@".to_string(),
            corner_radius: 8,
            border_height: 4,
            border_width: 4,
            line_spacing: 6,
            spacing: 8,
            back_color: "0xFFF6F0".to_string(),
            border_color: "0xF5E0CD".to_string(),
            text_color: "0x6E4D33".to_string(),
            candidate_text_color: "0x6E4D33".to_string(),
            comment_text_color: "0xAE937A".to_string(),
            hilited_text_color: "0xFFFFFF".to_string(),
            hilited_back_color: "0xD48E3B".to_string(),
            hilited_candidate_text_color: "0xFFFFFF".to_string(),
            hilited_candidate_back_color: "0xD48E3B".to_string(),
            custom_schemes: Vec::new(),
        }
    }

    #[test]
    fn appearance_merge_keeps_unknown_weasel_keys() {
        let existing = r#"
patch:
  "style/layout": horizontal
  "style/color_scheme": old_theme
  "preset_color_schemes/lost_temple/back_color": 0x111111
"#;
        let rendered =
            merge_weasel_custom(existing, &sample_appearance()).expect("merge appearance");

        assert!(rendered.contains("style/layout") || rendered.contains("layout:"));
        assert!(rendered.contains("lost_temple"));
        assert!(rendered.contains("Sarasa Gothic"));
        assert!(rendered.contains("rime_studio_blue"));
    }

    #[test]
    fn ice_settings_merge_keeps_unknown_keys() {
        let existing = r#"
patch:
  "translator/enable_user_dict": true
  "switches/@3/reset": 0
"#;
        let settings = RimeIceSettings {
            emoji: true,
            traditionalization: false,
            ascii_punct: false,
            full_shape: false,
            search_single_char: false,
            fuzzy_pinyin: false,
            traditional_preset: "s2t.json".to_string(),
        };
        let rendered =
            merge_rime_ice_custom(existing, &settings, LmdgPatchAction::Keep).expect("merge ice");

        assert!(rendered.contains("enable_user_dict"));
        assert!(rendered.contains("s2t.json"));
    }

    #[test]
    fn rejects_yaml_filename_traversal() {
        assert!(validate_config_relpath("default.custom.yaml").is_ok());
        assert!(validate_config_relpath("../default.custom.yaml").is_err());
        assert!(validate_config_relpath("sub/default.custom.yaml").is_ok());
        assert!(validate_config_relpath("lua/date.lua").is_ok());
        assert!(validate_config_relpath("custom_phrase.txt").is_ok());
        assert!(validate_config_relpath("default.custom.json").is_err());
        assert!(validate_config_relpath("").is_err());
    }

    #[test]
    fn rejects_dictionary_path_traversal() {
        let dir = env::temp_dir().join(format!(
            "rime-studio-dict-path-{}-{}",
            process::id(),
            timestamp()
        ));
        fs::create_dir_all(&dir).expect("create test dir");
        fs::write(dir.join("ok.dict.yaml"), "x").expect("write dict");

        assert!(validate_dictionary_path(&dir, "ok.dict.yaml").is_ok());
        assert!(validate_dictionary_path(&dir, "../ok.dict.yaml").is_err());
        assert!(validate_dictionary_path(&dir, "..\\secret.dict.yaml").is_err());
        assert!(validate_dictionary_path(&dir, "ok.txt").is_err());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn skips_compiled_and_sync_dictionary_dirs() {
        let root = PathBuf::from(r"C:\Users\test\AppData\Roaming\Rime");
        assert!(should_skip_dict_directory(&root.join("build"), &root));
        assert!(should_skip_dict_directory(&root.join("sync"), &root));
        assert!(should_skip_dict_directory(
            &root.join("luna_pinyin.userdb"),
            &root
        ));
        assert!(!should_skip_dict_directory(&root, &root));
        assert!(!should_skip_dict_directory(&root.join("cn_dicts"), &root));
    }

    #[test]
    fn backup_scope_describes_exclusions() {
        assert!(backup_scope_label("manual").contains("userdb"));
        assert!(backup_scope_label("before-save").contains("词库"));
    }

    #[test]
    fn paging_keys_use_yaml_bindings() {
        let contents = r#"
patch:
  key_binder/bindings:
    - {when: paging, accept: Up, send: Page_Up}
    - {when: has_menu, accept: Down, send: Page_Down}
"#;
        assert_eq!(detect_paging_keys(contents), "arrow_keys");
        assert_eq!(detect_navigation_keys(contents), "up_down");
    }

    #[test]
    fn detects_lmdg_grammar_patch() {
        assert!(has_lmdg_grammar_patch(
            "grammar:\n  language: wanxiang-lts-zh-hans\n"
        ));
        assert!(!has_lmdg_grammar_patch("patch:\n  emoji: true\n"));
    }
}
