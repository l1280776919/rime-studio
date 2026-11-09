use crate::backend::*;
use crate::RimeIceSettings;
use serde_yaml::Value;

fn settings(pairs: &[&str]) -> RimeIceSettings {
    RimeIceSettings {
        emoji: true,
        traditionalization: false,
        ascii_punct: false,
        full_shape: false,
        search_single_char: false,
        fuzzy_pinyin: !pairs.is_empty(),
        fuzzy_pairs: Some(pairs.iter().map(|pair| (*pair).into()).collect()),
        traditional_preset: "s2t.json".into(),
    }
}

fn save(existing: &str, settings: &RimeIceSettings) -> String {
    merge_rime_ice_custom(existing, settings, LmdgPatchAction::Keep).expect("save ice settings")
}

fn append_layouts(items: &str) -> Vec<String> {
    [
        "speller/algebra/+",
        "speller:\n    algebra/+",
        "speller/algebra:\n    +",
        "speller:\n    algebra:\n      +",
    ]
    .iter()
    .map(|path| format!("patch:\n  {path}: {items}\n  translator/enable_user_dict: false\n"))
    .collect()
}

fn patch(contents: &str) -> serde_yaml::Mapping {
    parse_yaml_mapping(contents)
        .expect("valid yaml")
        .get(yaml_str("patch"))
        .and_then(Value::as_mapping)
        .expect("patch mapping")
        .clone()
}

#[test]
fn fuzzy_detection_ignores_comments_other_paths_and_substrings() {
    let existing = r#"
# speller/algebra/+: [derive/^l/n/]
speller/algebra/+: [derive/^h/f/]
patch:
  translator/preedit_format: [derive/ang$/an/]
  speller/algebra: [derive/^zh/z/]
  speller/algebra/+:
    - xform/derive/^z/zh//custom/
    - derive/^g/k/
    - derive/^l/n/extra
    - {example: derive/^r/l/}
"#;
    assert!(detect_fuzzy_pinyin_pairs(existing).is_empty());
    let saved = save(existing, &settings(&[]));
    assert_eq!(patch(existing), {
        let mut saved_patch = patch(&saved);
        for path in [
            "switches/@1/reset",
            "switches/@2/reset",
            "switches/@3/reset",
            "switches/@4/reset",
            "switches/@5/reset",
            "traditionalize/opencc_config",
        ] {
            saved_patch.remove(yaml_str(path));
        }
        saved_patch
    });
}

#[test]
fn custom_only_append_survives_unrelated_ice_setting_saves_in_all_layouts() {
    for existing in append_layouts("['derive/^g/k/', 'xform/abc/def/', 42, {custom: keep}]") {
        let mut config = settings(&[]);
        config.emoji = false;
        config.traditional_preset = "s2tw.json".into();
        assert!(detect_fuzzy_pinyin_pairs(&existing).is_empty());
        let saved = save(&existing, &config);
        for (key, value) in patch(&existing) {
            assert_eq!(patch(&saved).get(&key), Some(&value));
        }
        assert_eq!(
            get_patch_path(&patch(&saved), "switches/@3/reset"),
            Some(&Value::from(0))
        );
        assert_eq!(save(&saved, &config), saved);
    }
}

#[test]
fn unrelated_save_preserves_one_way_rules_order_duplicates_and_comments() {
    let existing = r#"# my handwritten rules
patch:
  speller/algebra/+:
    - derive/^g/k/ # custom before managed
    - derive/^zh/z/ # deliberately one-way
    - derive/^zh/z/
    - xform/foo/bar/ # custom after managed
  switches/@3/reset: 1
"#;
    let mut config = settings(&["z_zh", "z_zh"]);
    config.emoji = false;
    let saved = save(existing, &config);
    assert!(saved.contains("- derive/^zh/z/ # deliberately one-way"));
    assert!(saved.contains("# custom before managed"));
    assert_eq!(
        get_patch_path(&patch(&saved), "speller/algebra/+"),
        get_patch_path(&patch(existing), "speller/algebra/+")
    );
    assert!(!saved.contains("derive/^z([^h])/zh$1/"));
    assert_eq!(save(&saved, &config), saved);
}

#[test]
fn reordered_pair_selection_does_not_rewrite_append() {
    for existing in append_layouts("['derive/^zh/z/', 'derive/ang$/an/', 'xform/foo/bar/']") {
        assert_eq!(detect_fuzzy_pinyin_pairs(&existing), vec!["z_zh", "an_ang"]);
        let saved = save(&existing, &settings(&["an_ang", "z_zh", "an_ang"]));
        for (key, value) in patch(&existing) {
            assert_eq!(patch(&saved).get(&key), Some(&value));
        }
    }
}

#[test]
fn enable_then_disable_preserves_custom_entries_in_all_layouts() {
    for existing in append_layouts("['derive/^g/k/', 'xform/foo/bar/', 42, {custom: keep}]") {
        let enabled = save(&existing, &settings(&["z_zh", "an_ang", "z_zh"]));
        assert_eq!(detect_fuzzy_pinyin_pairs(&enabled), vec!["z_zh", "an_ang"]);
        assert_eq!(enabled.matches("derive/^zh/z/").count(), 1);
        assert_eq!(save(&enabled, &settings(&["an_ang", "z_zh"])), enabled);
        let disabled = save(&enabled, &settings(&[]));
        assert!(detect_fuzzy_pinyin_pairs(&disabled).is_empty());
        for (key, value) in patch(&existing) {
            assert_eq!(patch(&disabled).get(&key), Some(&value));
        }
    }
}

#[test]
fn changing_pairs_preserves_unknown_rules_in_order_and_other_settings() {
    let existing = r#"
patch:
  speller/algebra/+:
    - derive/^g/k/
    - derive/^z([^h])/zh$1/
    - xform/foo/bar/
    - derive/^zh/z/
    - derive/ang$/an/custom/
  speller/algebra: [erase/^xx$/]
  grammar: {language: custom-language}
"#;
    let saved = save(existing, &settings(&["c_ch"]));
    let result = patch(&saved);
    let items = get_patch_path(&result, "speller/algebra/+")
        .expect("append")
        .as_sequence()
        .expect("list");
    assert_eq!(
        &items[..3],
        &[
            yaml_str("derive/^g/k/"),
            yaml_str("xform/foo/bar/"),
            yaml_str("derive/ang$/an/custom/"),
        ]
    );
    assert_eq!(detect_fuzzy_pinyin_pairs(&saved), vec!["c_ch"]);
    assert_eq!(
        get_patch_path(&result, "speller/algebra"),
        get_patch_path(&patch(existing), "speller/algebra")
    );
    assert_eq!(
        get_patch_path(&result, "grammar"),
        get_patch_path(&patch(existing), "grammar")
    );
}

#[test]
fn both_legacy_grouped_rules_can_be_changed_or_disabled() {
    for rule in ["derive/^([zcs])h/$1/", "derive/^([zcs])([^h])/$1h$2/"] {
        let existing = format!("patch:\n  speller/algebra/+: ['{rule}', 'derive/^g/k/']\n");
        assert_eq!(
            detect_fuzzy_pinyin_pairs(&existing),
            vec!["z_zh", "c_ch", "s_sh"]
        );
        for pairs in [vec!["c_ch"], vec![]] {
            let saved = save(&existing, &settings(&pairs));
            assert_eq!(detect_fuzzy_pinyin_pairs(&saved), pairs);
            assert!(!saved.contains(rule));
            assert!(saved.contains("derive/^g/k/"));
        }
    }
}

#[test]
fn legacy_single_pair_rules_are_recognized_exactly() {
    for (rule, pair) in [
        ("derive/^z/zh/", "z_zh"),
        ("derive/^c/ch/", "c_ch"),
        ("derive/^s/sh/", "s_sh"),
    ] {
        let existing = format!("patch:\n  speller/algebra/+: ['{rule}']\n");
        assert_eq!(detect_fuzzy_pinyin_pairs(&existing), vec![pair]);
        let saved = save(&existing, &settings(&[]));
        assert!(!saved.contains(rule));
        assert!(detect_fuzzy_pinyin_pairs(&saved).is_empty());
    }
}

#[test]
fn unsupported_append_nodes_are_preserved_or_rejected_without_overwrite() {
    for shape in [
        "custom-reference",
        "{custom: keep}",
        "{__include: other:/algebra}",
        "{__patch: other:/patch}",
        "{__append: ['derive/^l/n/']}",
    ] {
        for existing in append_layouts(shape) {
            let saved = save(&existing, &settings(&[]));
            for (key, value) in patch(&existing) {
                assert_eq!(patch(&saved).get(&key), Some(&value));
            }
            let error =
                merge_rime_ice_custom(&existing, &settings(&["c_ch"]), LmdgPatchAction::Keep)
                    .expect_err("do not overwrite advanced nodes");
            assert!(error.to_string().contains("speller/algebra/+"));
        }
    }
}

#[test]
fn conflicting_append_paths_survive_unrelated_saves_but_reject_edits() {
    let existing = r#"
patch:
  speller/algebra/+: [derive/^zh/z/]
  speller:
    algebra/+: [derive/^g/k/]
"#;
    let saved = save(existing, &settings(&["z_zh"]));
    for (key, value) in patch(existing) {
        assert_eq!(patch(&saved).get(&key), Some(&value));
    }
    assert!(merge_rime_ice_custom(existing, &settings(&[]), LmdgPatchAction::Keep).is_err());
}

#[test]
fn no_pair_field_and_enabled_empty_selection_keep_legacy_defaults() {
    for pairs in [None, Some(Vec::new())] {
        let mut config = settings(&[]);
        config.fuzzy_pinyin = true;
        config.fuzzy_pairs = pairs;
        let saved = save("", &config);
        for rule in fuzzy_pinyin_algebra_rules() {
            assert!(saved.contains(rule));
        }
        assert_eq!(patch(&save(&saved, &config)), patch(&saved));
    }
}

#[test]
fn empty_nested_append_does_not_remove_or_replace_its_parents() {
    for existing in append_layouts("['derive/^zh/z/']") {
        let saved = save(&existing, &settings(&[]));
        assert!(saved.contains("[]"));
        assert!(!saved.contains("{}"));
        assert_eq!(save(&saved, &settings(&[])), saved);
    }
}

#[test]
fn malformed_yaml_is_rejected_before_a_settings_write() {
    assert!(merge_rime_ice_custom("patch: [", &settings(&[]), LmdgPatchAction::Keep).is_err());
    assert!(merge_rime_ice_custom("patch: []", &settings(&[]), LmdgPatchAction::Keep).is_err());
}

#[test]
fn changing_another_pair_keeps_retained_rules_in_their_original_positions() {
    let existing = "patch:\n  speller/algebra/+: ['derive/^g/k/', 'derive/^zh/z/', 'xform/foo/bar/', 'derive/ang$/an/']\n";
    let saved = save(existing, &settings(&["z_zh", "c_ch"]));
    let result = patch(&saved);
    let items = get_patch_path(&result, "speller/algebra/+")
        .expect("append")
        .as_sequence()
        .expect("list");
    assert_eq!(
        &items[..3],
        &[
            yaml_str("derive/^g/k/"),
            yaml_str("derive/^zh/z/"),
            yaml_str("xform/foo/bar/"),
        ]
    );
    assert!(!saved.contains("derive/^z([^h])/zh$1/"));
    assert_eq!(detect_fuzzy_pinyin_pairs(&saved), vec!["z_zh", "c_ch"]);
}

#[test]
fn unknown_pair_identifiers_cannot_silently_remove_known_rules() {
    let existing = "patch:\n  speller/algebra/+: ['derive/^zh/z/']\n";
    assert!(
        merge_rime_ice_custom(existing, &settings(&["unknown"]), LmdgPatchAction::Keep).is_err()
    );
}

#[test]
fn null_append_can_be_enabled_without_modifying_siblings() {
    for existing in append_layouts("null") {
        let saved = save(&existing, &settings(&["c_ch"]));
        assert_eq!(detect_fuzzy_pinyin_pairs(&saved), vec!["c_ch"]);
        assert_eq!(
            get_patch_path(&patch(&saved), "translator/enable_user_dict"),
            Some(&Value::Bool(false))
        );
    }
}

#[test]
fn splitting_legacy_groups_preserves_both_directions_with_individual_overlap() {
    for (grouped, individual) in [
        ("derive/^([zcs])([^h])/$1h$2/", "derive/^ch/c/"),
        ("derive/^([zcs])h/$1/", "derive/^c([^h])/ch$1/"),
    ] {
        let existing =
            format!("patch:\n  speller/algebra/+: ['{grouped}', '{individual}', 'derive/^g/k/']\n");
        let saved = save(&existing, &settings(&["c_ch"]));
        assert_eq!(detect_fuzzy_pinyin_pairs(&saved), vec!["c_ch"]);
        for rule in rules_for_fuzzy_pairs(&["c_ch".into()]) {
            assert_eq!(saved.matches(rule).count(), 1);
        }
        assert!(!saved.contains(grouped));
        assert!(saved.contains("derive/^g/k/"));
        assert_eq!(save(&saved, &settings(&["c_ch"])), saved);
    }
}
