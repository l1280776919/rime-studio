use crate::backend::*;
use crate::QuickSettingsConfig;
use serde_yaml::Value;

const DEFAULT_BINDINGS: &str = r#"
key_binder:
  bindings:
    - {when: composing, accept: Tab, send: Shift+Right}
    - {when: composing, accept: Shift+Tab, send: Shift+Left}
    - {when: composing, accept: Alt+Left, send: Shift+Left}
    - {when: composing, accept: Alt+Right, send: Shift+Right}
    - {when: always, accept: Control+Shift+f, toggle: simplification}
"#;

// The shape written by Rime Studio before the fix, including a user's shortcut.
const LEGACY_CUSTOM: &str = r#"# keep my comment
patch:
  schema_list:
    - schema: rime_ice
  menu/page_size: 9
  ascii_composer/switch_key/Shift_L: commit_code
  ascii_composer/switch_key/Shift_R: commit_code
  key_binder/bindings:
    - {when: always, accept: Control+p, toggle: ascii_mode}
    - {when: paging, accept: Up, send: Page_Up}
    - {when: has_menu, accept: Down, send: Page_Down}
  unrelated: keep
"#;

fn quick() -> QuickSettingsConfig {
    QuickSettingsConfig {
        schema_list: vec!["rime_ice".into()],
        schema_id: "rime_ice".into(),
        page_size: 9,
        switch_key: "shift".into(),
        paging_keys: "comma_period".into(),
        navigation_keys: "up_down".into(),
        horizontal: true,
        inline_preedit: true,
    }
}

fn save(existing: &str, config: &QuickSettingsConfig) -> String {
    merge_default_custom(existing, config, &config.schema_list).expect("merge settings")
}

fn inherited_bindings() -> Vec<Value> {
    yaml_lookup(DEFAULT_BINDINGS, "key_binder/bindings")
        .expect("default bindings")
        .as_sequence()
        .expect("list")
        .clone()
}

// Model Rime's documented list patch semantics: a plain path replaces the list;
// /+ appends to it. Checking the effective result catches lost inherited keys.
fn effective_bindings(custom: &str) -> Vec<Value> {
    let replacement = yaml_lookup(custom, "key_binder/bindings");
    let mut bindings = replacement
        .as_ref()
        .map(|value| value.as_sequence().expect("replacement list").clone())
        .unwrap_or_else(inherited_bindings);
    if let Some(appended) = yaml_lookup(custom, "key_binder/bindings/+") {
        bindings.extend(appended.as_sequence().expect("append list").iter().cloned());
    }
    bindings
}

#[test]
fn first_quick_settings_save_preserves_inherited_shortcuts() {
    let saved = save("patch:\n  menu/page_size: 5\n", &quick());
    let effective = effective_bindings(&saved);
    assert!(effective.starts_with(&inherited_bindings()));
    assert!(effective.contains(&binding_value("paging", "comma", "Page_Up")));
    assert!(yaml_lookup(&saved, "key_binder/bindings").is_none());
    assert_eq!(save(&saved, &quick()), saved, "repeat saves must be stable");
}

#[test]
fn legacy_override_restores_inheritance_and_preserves_user_bindings() {
    let saved = save(LEGACY_CUSTOM, &quick());
    let effective = effective_bindings(&saved);
    assert!(effective.starts_with(&inherited_bindings()));
    let custom: Value =
        serde_yaml::from_str("{when: always, accept: Control+p, toggle: ascii_mode}")
            .expect("custom shortcut");
    assert!(effective.contains(&custom));
    assert!(!effective.contains(&binding_value("paging", "Up", "Page_Up")));
    assert!(saved.contains("# keep my comment"));
    assert_eq!(yaml_lookup(&saved, "unrelated"), Some(yaml_str("keep")));
    assert_eq!(save(&saved, &quick()), saved);
}

#[test]
fn nested_legacy_override_does_not_leave_a_parent_that_erases_inheritance() {
    for sibling in ["", "    select_first_character: bracketleft\n"] {
        let existing = format!(
            "patch:\n  schema_list: [{{schema: rime_ice}}]\n  menu:\n    page_size: 9\n  ascii_composer:\n    switch_key: {{Shift_L: noop, Shift_R: noop}}\n  key_binder:\n{sibling}    bindings:\n      - {{when: paging, accept: minus, send: Page_Up}}\n      - {{when: has_menu, accept: equal, send: Page_Down}}\n"
        );
        let saved = save(&existing, &quick());
        assert!(yaml_lookup(&saved, "key_binder").is_none());
        assert!(yaml_lookup(&saved, "key_binder/bindings").is_none());
        assert!(effective_bindings(&saved).starts_with(&inherited_bindings()));
        if !sibling.is_empty() {
            assert_eq!(
                yaml_lookup(&saved, "key_binder/select_first_character"),
                Some(yaml_str("bracketleft"))
            );
        }
        assert_eq!(save(&saved, &quick()), saved);
    }
}

#[test]
fn legacy_migration_keeps_manual_tab_overrides_and_existing_append_order() {
    let existing = format!(
        "{LEGACY_CUSTOM}  key_binder/bindings/+:\n    - {{when: composing, accept: Tab, send: Page_Down}}\n    - {{when: composing, accept: Shift+Tab, send: Page_Up}}\n    - {{when: always, accept: Control+t, send_sequence: abc}}\n"
    );
    let saved = save(&existing, &quick());
    let effective = effective_bindings(&saved);
    let tab = binding_value("composing", "Tab", "Page_Down");
    let backtab = binding_value("composing", "Shift+Tab", "Page_Up");
    assert!(effective.starts_with(&inherited_bindings()));
    assert!(effective.contains(&tab));
    assert!(effective.contains(&backtab));
    // Rime gives later bindings with the same condition precedence.
    assert_eq!(
        effective.iter().rev().find(|binding| {
            binding
                .as_mapping()
                .and_then(|m| mapping_string(m, "accept"))
                .as_deref()
                == Some("Tab")
        }),
        Some(&tab)
    );
    assert!(saved.contains("send_sequence: abc"));
    assert_eq!(save(&saved, &quick()), saved);
}

#[test]
fn explicit_user_replacement_is_not_mistaken_for_a_legacy_override() {
    let mut saved =
        "patch:\n  key_binder/bindings:\n    - {when: composing, accept: Tab, send: Page_Down}\n"
            .to_string();
    for _ in 0..3 {
        saved = save(&saved, &quick());
        assert_eq!(
            yaml_lookup(&saved, "key_binder/bindings"),
            Some(Value::Sequence(vec![binding_value(
                "composing",
                "Tab",
                "Page_Down"
            )]))
        );
        assert!(!effective_bindings(&saved).contains(&binding_value(
            "composing",
            "Tab",
            "Shift+Right"
        )));
    }
}

#[test]
fn explicit_empty_and_included_lists_keep_their_original_meaning() {
    for replacement in ["[]", "null", "{__include: my_keys:/bindings}"] {
        let mut saved = format!("patch:\n  key_binder/bindings: {replacement}\n");
        let original = yaml_lookup(&saved, "key_binder/bindings");
        for _ in 0..2 {
            saved = save(&saved, &quick());
            assert_eq!(yaml_lookup(&saved, "key_binder/bindings"), original);
            assert!(yaml_lookup(&saved, "key_binder/bindings/+").is_some());
        }
    }
}

#[test]
fn appended_settings_round_trip_without_stale_managed_bindings() {
    let mut saved = LEGACY_CUSTOM.to_string();
    for paging in ["arrow_keys", "minus_equal", "comma_period", "arrow_keys"] {
        for navigation in ["left_right", "up_down"] {
            let mut config = quick();
            config.paging_keys = paging.into();
            config.navigation_keys = navigation.into();
            saved = save(&saved, &config);
            assert_eq!(detect_paging_keys(&saved), paging);
            assert_eq!(detect_navigation_keys(&saved), navigation);
            let effective = effective_bindings(&saved);
            assert!(effective.starts_with(&inherited_bindings()));
            assert_eq!(
                effective
                    .into_iter()
                    .filter(is_managed_binding)
                    .collect::<Vec<_>>(),
                managed_key_bindings(&config)
            );
            assert_eq!(save(&saved, &config), saved);
        }
    }
}

#[test]
fn malformed_append_patch_is_rejected_instead_of_discarded() {
    for value in ["important", "{__include: other:/bindings}"] {
        let existing = format!("patch:\n  key_binder/bindings/+: {value}\n");
        assert!(merge_default_custom(&existing, &quick(), &["rime_ice".into()]).is_err());
    }
}
