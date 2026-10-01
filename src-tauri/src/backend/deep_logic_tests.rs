use crate::backend::*;
use crate::*;
use std::{env, fs, path::Path, process::Command};

// Isolate APPDATA/LOCALAPPDATA without changing the environment of parallel tests.
fn isolated(name: &str, run: impl FnOnce()) {
    if env::var("RIME_DEEP_TEST").as_deref() == Ok(name) {
        run();
        return;
    }
    let root = env::temp_dir().join(format!("rime-deep-{}-{name}", std::process::id()));
    fs::create_dir_all(root.join("roaming/Rime")).expect("create fixture");
    fs::create_dir_all(root.join("local")).expect("create fixture");
    let output = Command::new(env::current_exe().expect("test executable"))
        .arg(format!("backend::deep_logic_tests::{name}"))
        .arg("--exact")
        .arg("--test-threads=1")
        .env("RIME_DEEP_TEST", name)
        .env("APPDATA", root.join("roaming"))
        .env("LOCALAPPDATA", root.join("local"))
        .output()
        .expect("run isolated test");
    fs::remove_dir_all(root).expect("cleanup");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn quick() -> QuickSettingsConfig {
    QuickSettingsConfig {
        schema_list: Vec::new(),
        schema_id: "rime_ice".into(),
        page_size: 9,
        switch_key: "shift".into(),
        paging_keys: "comma_period".into(),
        navigation_keys: "left_right".into(),
        horizontal: true,
        inline_preedit: true,
    }
}

#[test]
fn quick_settings_preserve_schemas_and_validate_both_files_before_writing() {
    isolated(
        "quick_settings_preserve_schemas_and_validate_both_files_before_writing",
        || {
            let user = rime_user_dir().expect("user directory");
            let original = "patch:\n  schema_list:\n    - schema: rime_ice\n    - schema: luna_pinyin\n  menu/page_size: 5\n";
            fs::write(user.join("default.custom.yaml"), original).expect("fixture");
            fs::write(user.join("weasel.custom.yaml"), "patch: [").expect("fixture");
            assert!(save_quick_settings_sync(quick()).is_err());
            assert_eq!(
                fs::read_to_string(user.join("default.custom.yaml")).expect("read"),
                original
            );
            fs::write(user.join("weasel.custom.yaml"), "patch: {}\n").expect("fixture");
            let preview = preview_quick_settings_sync(quick()).expect("preview");
            assert!(!preview.files[0]
                .diff_lines
                .iter()
                .any(|line| line.contains("- schema: luna_pinyin") && line.starts_with('-')));
            save_quick_settings_sync(quick()).expect("save");
            let saved = fs::read_to_string(user.join("default.custom.yaml")).expect("read");
            assert_eq!(parse_schema_list(&saved), vec!["rime_ice", "luna_pinyin"]);
            assert_eq!(detect_paging_keys(&saved), "comma_period");
            assert_eq!(detect_navigation_keys(&saved), "left_right");
            let bindings = yaml_lookup(&saved, "key_binder/bindings").expect("bindings");
            let bindings = bindings.as_sequence().expect("sequence");
            assert!(bindings.contains(&binding_value("has_menu", "Left", "Up")));
            assert!(bindings.contains(&binding_value("paging", "comma", "Page_Up")));
            let mut config = quick();
            config.paging_keys = "minus_equal".into();
            save_quick_settings_sync(config).expect("save minus paging");
            let saved = fs::read_to_string(user.join("default.custom.yaml")).expect("read");
            assert_eq!(detect_paging_keys(&saved), "minus_equal");
            let before = saved;
            let mut invalid = quick();
            invalid.schema_id = "../other".into();
            assert!(save_quick_settings_sync(invalid).is_err());
            assert_eq!(
                fs::read_to_string(user.join("default.custom.yaml")).expect("read"),
                before
            );
        },
    );
}

#[test]
fn dictionary_import_changes_preserve_header_entries_and_reference_order() {
    isolated(
        "dictionary_import_changes_preserve_header_entries_and_reference_order",
        || {
            let user = rime_user_dir().expect("user");
            fs::write(
                user.join("default.custom.yaml"),
                "patch:\n  schema_list: [{schema: example}]\n",
            )
            .expect("fixture");
            fs::write(
                user.join("example.schema.yaml"),
                "schema: {name: example}\ntranslator: {dictionary: base}\n",
            )
            .expect("fixture");
            fs::write(
                user.join("example.custom.yaml"),
                "patch: {translator/dictionary: custom}\n",
            )
            .expect("fixture");
            let original = "---\nname: custom\nversion: 'original'\nsort: by_weight\ncolumns: [text, code, weight]\nuse_preset_vocabulary: false\nimport_tables: ['missing', 'present']\n...\n原词条\tyuan ci tiao\t42\n";
            fs::write(user.join("custom.dict.yaml"), original).expect("fixture");
            fs::write(
                user.join("present.dict.yaml"),
                "---\nname: present\n...\n现有\txian you\t1\n",
            )
            .expect("fixture");
            let cfg = read_dictionary_config_sync().expect("read configuration");
            assert_eq!(cfg.main_dictionary.as_deref(), Some("custom"));
            assert_eq!(cfg.imports, vec!["missing", "present"]);
            add_dictionary_to_current_schema_sync("new".into()).expect("add reference");
            let saved = fs::read_to_string(user.join("custom.dict.yaml")).expect("read");
            assert_eq!(
                split_dictionary_header(&saved).1,
                split_dictionary_header(original).1
            );
            assert_eq!(
                parse_import_tables(&saved),
                vec!["missing", "present", "new"]
            );
            let header =
                parse_yaml_mapping(split_dictionary_header(&saved).0).expect("parse header");
            assert_eq!(header.get(yaml_str("version")), Some(&yaml_str("original")));
            assert_eq!(
                header.get(yaml_str("use_preset_vocabulary")),
                Some(&serde_yaml::Value::Bool(false))
            );
            assert!(header.contains_key(yaml_str("columns")));
            assert!(save_dictionary_imports_sync(vec!["custom".into()]).is_err());
            assert!(save_dictionary_imports_sync(vec!["C:/outside".into()]).is_err());
            assert_eq!(
                fs::read_to_string(user.join("custom.dict.yaml")).expect("read"),
                saved
            );
            remove_dictionary_from_current_schema_sync("present".into()).expect("remove reference");
            assert_eq!(
                parse_import_tables(
                    &fs::read_to_string(user.join("custom.dict.yaml")).expect("read")
                ),
                vec!["missing", "new"]
            );
        },
    );
}

#[test]
fn nested_snapshots_restore_lua_and_dictionaries_with_complete_safety_backup() {
    isolated(
        "nested_snapshots_restore_lua_and_dictionaries_with_complete_safety_backup",
        || {
            let user = rime_user_dir().expect("user");
            fs::create_dir_all(user.join("lua/tools")).expect("fixture");
            fs::create_dir_all(user.join("dicts")).expect("fixture");
            fs::create_dir_all(user.join("build")).expect("fixture");
            fs::write(user.join("lua/tools/date.lua"), "old lua").expect("fixture");
            fs::write(user.join("dicts/extra.dict.yaml"), "old dict").expect("fixture");
            fs::write(user.join("build/generated.dict.yaml"), "generated").expect("fixture");
            fs::write(user.join("installation.yaml"), "installation_id: old").expect("fixture");
            let snapshot = backup_user_config(&user, BackupKind::Manual).expect("snapshot");
            assert!(snapshot.join("lua/tools/date.lua").is_file());
            assert!(snapshot.join("dicts/extra.dict.yaml").is_file());
            assert!(!snapshot.join("build").exists());
            let auto = backup_user_config(&user, BackupKind::BeforeSave).expect("automatic backup");
            assert!(auto.join("dicts/extra.dict.yaml").is_file());
            fs::write(user.join("lua/tools/date.lua"), "current lua").expect("fixture");
            fs::write(user.join("installation.yaml"), "installation_id: current").expect("fixture");
            let result = restore_backup_dir(&user, &snapshot).expect("restore");
            assert_eq!(result.restored_files, 3);
            assert_eq!(
                fs::read_to_string(user.join("lua/tools/date.lua")).expect("read"),
                "old lua"
            );
            let safety = std::path::PathBuf::from(&result.safety_backup_dir);
            assert_eq!(
                fs::read_to_string(safety.join("lua/tools/date.lua")).expect("read safety"),
                "current lua"
            );
            assert_eq!(
                fs::read_to_string(safety.join("installation.yaml")).expect("read safety"),
                "installation_id: current"
            );
            let name = snapshot
                .file_name()
                .expect("filename")
                .to_string_lossy()
                .to_string();
            let preview = preview_backup_sync(name.clone()).expect("preview");
            assert!(preview
                .files
                .iter()
                .any(|file| file.name == "dicts/extra.dict.yaml"));
            assert_eq!(
                list_backup_dirs(&user)
                    .expect("list")
                    .iter()
                    .find(|entry| entry.name == name)
                    .expect("snapshot")
                    .files,
                3
            );
        },
    );
}

#[test]
fn dictionary_editor_accepts_header_plus_rows_and_import_backs_up_overwrites() {
    isolated(
        "dictionary_editor_accepts_header_plus_rows_and_import_backs_up_overwrites",
        || {
            let user = rime_user_dir().expect("user");
            let old = "---\nname: custom\n...\n旧词\tjiu ci\t2\n";
            write_config_file_content_sync("custom.dict.yaml".into(), old.into())
                .expect("edit dictionary");
            import_dictionary_sync(
                "custom.dict.yaml".into(),
                "新词\txin ci\t3\n".as_bytes().to_vec(),
            )
            .expect("import");
            assert!(list_backup_dirs(&user).expect("backups").iter().any(
                |entry| fs::read_to_string(Path::new(&entry.path).join("custom.dict.yaml"))
                    .ok()
                    .as_deref()
                    == Some(old)
            ));
            let saved = fs::read_to_string(user.join("custom.dict.yaml")).expect("read");
            assert!(write_config_file_content_sync(
                "custom.dict.yaml".into(),
                "---\nname: [\n...\n词\tci\t1\n".into()
            )
            .is_err());
            assert_eq!(
                fs::read_to_string(user.join("custom.dict.yaml")).expect("read"),
                saved
            );
        },
    );
}

#[test]
fn sync_config_rejects_invalid_yaml_and_paths_without_overwriting() {
    isolated(
        "sync_config_rejects_invalid_yaml_and_paths_without_overwriting",
        || {
            let user = rime_user_dir().expect("user");
            let path = user.join("installation.yaml");
            fs::write(&path, "installation_id: [").expect("fixture");
            assert!(save_sync_config_sync(Some("new".into()), None).is_err());
            assert_eq!(
                fs::read_to_string(&path).expect("read"),
                "installation_id: ["
            );
            fs::write(&path, "installation_id: original\ncustom: keep\n").expect("fixture");
            assert!(save_sync_config_sync(Some("../outside".into()), None).is_err());
            save_sync_config_sync(Some("new".into()), None).expect("save");
            assert_eq!(
                parse_string_after_key(&fs::read_to_string(&path).expect("read"), "custom"),
                Some("keep".into())
            );
            assert!(list_userdb_entries_sync("../installation.yaml".into(), 10, 0, None).is_err());
        },
    );
}

fn directory_link(target: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).expect("create directory link");
    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()
            .expect("create junction");
        assert!(status.success());
    }
}

#[test]
fn linked_directories_cannot_escape_user_root_or_loop_during_scans() {
    isolated(
        "linked_directories_cannot_escape_user_root_or_loop_during_scans",
        || {
            let user = rime_user_dir().expect("user");
            let outside = user.parent().expect("parent").join("outside");
            fs::create_dir_all(&outside).expect("fixture");
            fs::write(outside.join("secret.dict.yaml"), "external").expect("fixture");
            directory_link(&outside, &user.join("linked"));
            directory_link(&user, &user.join("loop"));
            assert!(resolve_user_relative_path(&user, "linked/new/file.yaml", false).is_err());
            assert!(validate_dictionary_path(&user, "linked/secret.dict.yaml").is_err());
            assert!(write_config_file_content_sync(
                "linked/new/file.yaml".into(),
                "test: 1".into()
            )
            .is_err());
            assert!(list_yaml_config_files_sync().expect("scan").is_empty());
            assert!(list_dictionaries_sync().expect("scan").is_empty());
            assert!(collect_snapshot_files(&user)
                .expect("snapshot scan")
                .is_empty());
        },
    );
}

#[test]
fn user_schema_overrides_system_and_patch_only_files_are_not_installed_schemas() {
    isolated(
        "user_schema_overrides_system_and_patch_only_files_are_not_installed_schemas",
        || {
            let user = rime_user_dir().expect("user");
            fs::write(user.join("orphan.custom.yaml"), "patch: {}\n").expect("fixture");
            fs::write(
                user.join("demo.schema.yaml"),
                "schema: {name: Demo, schema_id: demo}\ntranslator: {dictionary: demo}\n",
            )
            .expect("fixture");
            let system = app_data_dir().expect("app").join("fake-weasel");
            fs::create_dir_all(system.join("data")).expect("fixture");
            let deployer = system.join("WeaselDeployer.exe");
            fs::write(&deployer, "MZ").expect("fixture");
            fs::write(
                system.join("data/demo.schema.yaml"),
                "schema: {name: SystemDemo, schema_id: demo}\n",
            )
            .expect("fixture");
            fs::write(
                app_data_dir().expect("app").join("weasel-deployer.txt"),
                deployer.to_string_lossy().as_bytes(),
            )
            .expect("fixture");
            let list = list_schemas_sync().expect("schemas");
            let demo = list
                .iter()
                .find(|schema| schema.id == "demo")
                .expect("user schema");
            assert_eq!(demo.name, "Demo");
            assert_eq!(
                demo.path,
                user.join("demo.schema.yaml").display().to_string()
            );
            assert!(!list.iter().any(|schema| schema.id == "orphan"));
            copy_schema_sync("demo".into()).expect("create customization");
            let copied = fs::read_to_string(user.join("demo.custom.yaml")).expect("read");
            let root = parse_yaml_mapping(&copied).expect("parse");
            assert!(root
                .get(yaml_str("patch"))
                .expect("patch")
                .as_mapping()
                .expect("mapping")
                .is_empty());
            assert!(!root.contains_key(yaml_str("schema")));
        },
    );
}

#[test]
fn invalid_grammar_patch_does_not_delete_installed_model() {
    isolated(
        "invalid_grammar_patch_does_not_delete_installed_model",
        || {
            let user = rime_user_dir().expect("user");
            fs::write(user.join("wanxiang-lts-zh-hans.gram"), "model").expect("fixture");
            fs::write(user.join("rime_ice.custom.yaml"), "patch: [").expect("fixture");
            assert!(uninstall_lmdg_grammar_sync().is_err());
            assert_eq!(
                fs::read_to_string(user.join("wanxiang-lts-zh-hans.gram")).expect("read"),
                "model"
            );
        },
    );
}

#[test]
fn paths_reject_windows_prefixes_streams_devices_and_parent_components() {
    for path in [
        "../file.yaml",
        "/file.yaml",
        "C:/file.yaml",
        "file.yaml:stream.yaml",
        "\\\\server\\file.yaml",
        "CON.yaml",
        "lua/../outside.lua",
        "file.yaml ",
        "lua//file.lua",
    ] {
        assert!(!valid_user_relative_path(path), "{path}");
    }
    assert!(valid_user_relative_path("lua/中文脚本.lua"));
    assert!(sanitize_schema_id("../rime_ice").is_err());
    assert!(sanitize_schema_id("C:outside").is_err());
}

#[test]
fn scel_unknown_pinyin_index_is_not_silently_removed() {
    let mut data = vec![0u8; 0x1540];
    data.extend(1u16.to_le_bytes());
    data.extend(0u16.to_le_bytes());
    data.extend(0u16.to_le_bytes());
    data.extend(2u16.to_le_bytes());
    data.extend([b'a', 0]);
    data.extend(1u16.to_le_bytes());
    data.extend(4u16.to_le_bytes());
    data.extend(0u16.to_le_bytes());
    data.extend(99u16.to_le_bytes());
    data.extend(4u16.to_le_bytes());
    data.extend("阿安".encode_utf16().flat_map(u16::to_le_bytes));
    data.extend(0u16.to_le_bytes());
    assert!(parse_scel_entries(&data).is_err());
    assert_eq!(read_u16_le(&data, usize::MAX), None);
}

#[test]
fn interrupted_empty_and_truncated_downloads_preserve_the_previous_file() {
    isolated(
        "interrupted_empty_and_truncated_downloads_preserve_the_previous_file",
        || {
            let user = rime_user_dir().expect("user");
            let path = user.join("model.gram");
            fs::write(&path, "previous model").expect("fixture");
            for (bytes, size, limit) in [
                (b"new".as_slice(), Some(10), 100),
                (b"".as_slice(), None, 100),
                (b"oversized".as_slice(), None, 3),
            ] {
                assert!(download_reader_to_file(
                    bytes,
                    &path,
                    limit,
                    size,
                    "empty",
                    "too large",
                    |_, _| {}
                )
                .is_err());
                assert_eq!(fs::read_to_string(&path).expect("read"), "previous model");
            }
            struct Interrupted;
            impl std::io::Read for Interrupted {
                fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "interrupted",
                    ))
                }
            }
            assert!(download_reader_to_file(
                Interrupted,
                &path,
                100,
                None,
                "empty",
                "too large",
                |_, _| {}
            )
            .is_err());
            assert_eq!(fs::read_to_string(&path).expect("read"), "previous model");
            download_reader_to_file(
                b"complete".as_slice(),
                &path,
                100,
                Some(8),
                "empty",
                "too large",
                |_, _| {},
            )
            .expect("download");
            assert_eq!(fs::read_to_string(&path).expect("read"), "complete");
            assert_eq!(fs::read_dir(&user).expect("list").count(), 1);
        },
    );
}

#[test]
fn phrases_round_trip_spaces_empty_codes_and_metadata() {
    isolated("phrases_round_trip_spaces_empty_codes_and_metadata", || {
        let user = rime_user_dir().expect("user");
        let path = user.join("custom_phrase.txt");
        fs::write(
            &path,
            "\u{feff}# header\n---\n...\n  spaced phrase  \t\t12\n# interior note\n你好\tnh\t4\n",
        )
        .expect("fixture");
        let phrases = get_custom_phrases_sync().expect("read");
        assert_eq!(phrases.len(), 2);
        assert_eq!(phrases[0].text, "  spaced phrase  ");
        assert_eq!(phrases[0].code, "");
        assert_eq!(phrases[0].weight, 12);
        save_custom_phrases_sync(phrases).expect("save");
        let saved = fs::read_to_string(path).expect("read");
        assert!(saved.contains("# interior note"));
        assert!(saved.contains("  spaced phrase  \t\t12"));
        assert_eq!(get_custom_phrases_sync().expect("read").len(), 2);
    });
}

#[test]
fn invalid_phrase_fields_and_unreadable_files_do_not_overwrite() {
    isolated(
        "invalid_phrase_fields_and_unreadable_files_do_not_overwrite",
        || {
            let user = rime_user_dir().expect("user");
            let path = user.join("custom_phrase.txt");
            let original = "old\tcode\t1\n";
            fs::write(&path, original).expect("fixture");
            for (text, code) in [
                ("new\nother", "a"),
                ("new", "a\tb"),
                ("", "a"),
                ("# comment", "a"),
                ("...", "a"),
            ] {
                assert!(save_custom_phrases_sync(vec![PhraseEntry {
                    text: text.into(),
                    code: code.into(),
                    weight: 1
                }])
                .is_err());
                assert_eq!(fs::read_to_string(&path).expect("read"), original);
            }
            fs::write(&path, [0xff]).expect("fixture");
            assert!(save_custom_phrases_sync(Vec::new()).is_err());
            assert_eq!(fs::read(&path).expect("read"), vec![0xff]);
            fs::write(&path, "phrase\tcode\tnot-a-weight").expect("fixture");
            assert!(get_custom_phrases_sync().is_err());
        },
    );
}

#[test]
fn lua_toggle_preserves_unrelated_exports_and_long_comments() {
    isolated(
        "lua_toggle_preserves_unrelated_exports_and_long_comments",
        || {
            let user = rime_user_dir().expect("user");
            fs::create_dir_all(user.join("lua")).expect("directory");
            fs::write(user.join("lua/date.lua"), "return function() end").expect("fixture");
            let original = "local update = true\nother = require(\"date\")\n--[=[\ndate_translator = require(\"date\")\n]=]\n";
            fs::write(user.join("rime.lua"), original).expect("fixture");
            assert!(!list_lua_plugins_sync().expect("list")[0].enabled);
            let plugins = toggle_lua_plugin_sync("date".into(), true).expect("enable");
            assert!(plugins[0].enabled);
            let saved = fs::read_to_string(user.join("rime.lua")).expect("read");
            assert!(saved.starts_with(original));
            assert!(!toggle_lua_plugin_sync("date".into(), false).expect("disable")[0].enabled);
            assert!(fs::read_to_string(user.join("rime.lua"))
                .expect("read")
                .starts_with(original));
        },
    );
}

#[test]
fn lua_disable_does_not_install_and_failed_reads_preserve_files() {
    isolated(
        "lua_disable_does_not_install_and_failed_reads_preserve_files",
        || {
            let user = rime_user_dir().expect("user");
            toggle_lua_plugin_sync("date".into(), false).expect("disable");
            assert!(!user.join("lua").exists());
            assert!(!user.join("rime.lua").exists());
            fs::write(user.join("rime.lua"), [0xff]).expect("fixture");
            assert!(toggle_lua_plugin_sync("date".into(), true).is_err());
            assert!(!user.join("lua").exists());
            assert_eq!(fs::read(user.join("rime.lua")).expect("read"), vec![0xff]);
            fs::remove_file(user.join("rime.lua")).expect("remove");
            save_lua_script_content_sync("date".into(), "original".into()).expect("save");
            save_lua_script_content_sync("date".into(), "edited".into()).expect("save");
            let backups = list_backup_dirs(&user).expect("backups");
            assert!(backups.iter().any(|backup| {
                let snapshot = Path::new(&backup.path).join("lua/date.lua");
                fs::read_to_string(snapshot).ok().as_deref() == Some("original")
            }));
            fs::write(user.join("lua/date.lua"), [0xff]).expect("fixture");
            fs::write(
                user.join("rime.lua"),
                "date_translator = require(\"date\")\n",
            )
            .expect("fixture");
            assert!(toggle_lua_plugin_sync("date".into(), true).is_err());
            assert!(
                !toggle_lua_plugin_sync("date".into(), false).expect("disable damaged script")[0]
                    .enabled
            );
            assert_eq!(
                fs::read(user.join("lua/date.lua")).expect("read"),
                vec![0xff]
            );
        },
    );
}

#[test]
fn lua_and_phrase_operations_reject_linked_paths_outside_user_root() {
    isolated(
        "lua_and_phrase_operations_reject_linked_paths_outside_user_root",
        || {
            let user = rime_user_dir().expect("user");
            let outside = user.parent().expect("parent").join("outside-lua");
            fs::create_dir_all(&outside).expect("directory");
            fs::write(outside.join("date.lua"), "private").expect("fixture");
            directory_link(&outside, &user.join("lua"));
            directory_link(&outside, &user.join("custom_phrase.txt"));
            assert!(get_custom_phrases_sync().is_err());
            assert!(save_custom_phrases_sync(Vec::new()).is_err());
            assert!(get_lua_script_content_sync("date".into()).is_err());
            assert!(save_lua_script_content_sync("date".into(), "overwrite".into()).is_err());
            assert!(toggle_lua_plugin_sync("date".into(), true).is_err());
            assert_eq!(
                fs::read_to_string(outside.join("date.lua")).expect("read"),
                "private"
            );
        },
    );
}

#[test]
fn settings_reads_reject_corrupt_files_without_repairing_from_defaults() {
    isolated(
        "settings_reads_reject_corrupt_files_without_repairing_from_defaults",
        || {
            let user = rime_user_dir().expect("user");
            let default = "patch:\n  schema_list:\n    - schema: z_last\n    - schema: a_first\n    - schema: missing\n";
            fs::write(user.join("default.custom.yaml"), default).expect("fixture");
            fs::write(user.join("weasel.custom.yaml"), "patch: {}\n").expect("fixture");
            assert_eq!(
                get_quick_settings_sync().expect("read").schema_list,
                vec!["z_last", "a_first", "missing"]
            );
            fs::write(user.join("weasel.custom.yaml"), "patch: [").expect("fixture");
            assert!(get_quick_settings_sync().is_err());
            assert!(get_appearance_config_sync().is_err());
            assert!(repair_config_health_sync().is_err());
            assert_eq!(
                fs::read_to_string(user.join("default.custom.yaml")).expect("read"),
                default
            );
            assert_eq!(
                fs::read_to_string(user.join("weasel.custom.yaml")).expect("read"),
                "patch: ["
            );
            fs::write(user.join("rime_ice.custom.yaml"), [0xff]).expect("fixture");
            assert!(get_rime_ice_settings_sync().is_err());
        },
    );
}

#[test]
fn settings_and_backup_commands_reject_external_directory_links() {
    isolated(
        "settings_and_backup_commands_reject_external_directory_links",
        || {
            let user = rime_user_dir().expect("user");
            let outside = user.parent().expect("parent").join("outside-settings");
            fs::create_dir_all(&outside).expect("directory");
            fs::write(outside.join("private.yaml"), "private").expect("fixture");
            directory_link(&outside, &user.join("weasel.custom.yaml"));
            assert!(save_quick_settings_sync(quick()).is_err());
            assert!(get_appearance_config_sync().is_err());
            let app = app_data_dir().expect("app");
            fs::create_dir_all(&app).expect("directory");
            directory_link(&outside, &app.join("backup-rime-studio-manual-external"));
            assert!(validated_backup_dir(&user, "backup-rime-studio-manual-external").is_err());
            assert!(delete_backup_sync("backup-rime-studio-manual-external".into()).is_err());
            assert!(list_backups_sync().expect("list").is_empty());
            let original = app.join("backup-rime-studio-manual-original");
            fs::create_dir_all(&original).expect("directory");
            fs::write(original.join("default.custom.yaml"), "patch: {}\n").expect("fixture");
            directory_link(&original, &app.join("backup-rime-studio-manual-alias"));
            assert!(delete_backup_sync("backup-rime-studio-manual-alias".into()).is_err());
            assert!(original.join("default.custom.yaml").is_file());
            assert_eq!(
                fs::read_to_string(outside.join("private.yaml")).expect("read"),
                "private"
            );
        },
    );
}

#[test]
fn installer_failures_preserve_the_previous_complete_executable() {
    isolated(
        "installer_failures_preserve_the_previous_complete_executable",
        || {
            let user = rime_user_dir().expect("user");
            let path = user.join("setup.exe");
            fs::write(&path, "MZprevious executable").expect("fixture");
            for (body, total) in [
                (b"MZpartial".as_slice(), Some(100)),
                (b"<html>error".as_slice(), None),
                (b"".as_slice(), None),
            ] {
                assert!(save_installer(body, &path, total, |_, _| {}).is_err());
                assert_eq!(
                    fs::read_to_string(&path).expect("read"),
                    "MZprevious executable"
                );
            }
            save_installer(b"MZcomplete".as_slice(), &path, Some(10), |_, _| {}).expect("download");
            assert_eq!(fs::read_to_string(path).expect("read"), "MZcomplete");
            assert_eq!(fs::read_dir(user).expect("list").count(), 1);
        },
    );
}

#[test]
fn setting_writers_wait_for_the_configuration_operation_lock() {
    isolated(
        "setting_writers_wait_for_the_configuration_operation_lock",
        || {
            use std::{
                sync::{mpsc, Arc, Barrier},
                thread,
                time::Duration,
            };
            let guard = lock_config_write().expect("lock");
            let barrier = Arc::new(Barrier::new(2));
            let worker_barrier = barrier.clone();
            let (sender, receiver) = mpsc::channel();
            let worker = thread::spawn(move || {
                worker_barrier.wait();
                sender
                    .send(save_quick_settings_sync(quick()))
                    .expect("result");
            });
            barrier.wait();
            assert!(matches!(
                receiver.recv_timeout(Duration::from_millis(150)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
            drop(guard);
            receiver
                .recv_timeout(Duration::from_secs(10))
                .expect("completion")
                .expect("save");
            worker.join().expect("worker");
        },
    );
}

#[test]
fn creating_a_snapshot_waits_for_backup_operations_to_finish() {
    isolated(
        "creating_a_snapshot_waits_for_backup_operations_to_finish",
        || {
            use std::{
                sync::{mpsc, Arc, Barrier},
                thread,
                time::Duration,
            };
            let user = rime_user_dir().expect("user");
            fs::write(user.join("default.custom.yaml"), "patch: {}\n").expect("fixture");
            let guard = lock_backup_operation().expect("lock");
            let barrier = Arc::new(Barrier::new(2));
            let worker_barrier = barrier.clone();
            let (sender, receiver) = mpsc::channel();
            let worker = thread::spawn(move || {
                worker_barrier.wait();
                sender
                    .send(backup_user_config(&user, BackupKind::BeforeSave))
                    .expect("result");
            });
            barrier.wait();
            assert!(matches!(
                receiver.recv_timeout(Duration::from_millis(150)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ));
            drop(guard);
            let backup = receiver
                .recv_timeout(Duration::from_secs(10))
                .expect("completion")
                .expect("backup");
            assert!(backup.join("default.custom.yaml").is_file());
            worker.join().expect("worker");
        },
    );
}

#[test]
fn quick_settings_do_not_materialize_or_overwrite_appearance_defaults() {
    isolated(
        "quick_settings_do_not_materialize_or_overwrite_appearance_defaults",
        || {
            let user = rime_user_dir().expect("user");
            fs::write(user.join("weasel.custom.yaml"), "patch:\n  style/color_scheme: native_theme\n  style/font_point: 19\n  style/layout/corner_radius: 23\n").expect("fixture");
            save_quick_settings_sync(quick()).expect("save");
            let saved = fs::read_to_string(user.join("weasel.custom.yaml")).expect("read");
            assert_eq!(
                parse_string_after_key(&saved, "style/color_scheme").as_deref(),
                Some("native_theme")
            );
            assert_eq!(parse_u32_after_key(&saved, "style/font_point"), Some(19));
            assert!(yaml_lookup(&saved, "preset_color_schemes").is_none());
            assert!(yaml_lookup(&saved, "style/corner_radius").is_none());
            assert_eq!(
                parse_u32_after_key(&saved, "style/layout/corner_radius"),
                Some(23)
            );
        },
    );
}
