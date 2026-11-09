use crate::backend::{parse_yaml_mapping, yaml_str};
use crate::{RimeError, RimeIceSettings};
use serde_yaml::{Mapping, Value};

const FUZZY_PAIRS: &[&str] = &[
    "z_zh", "c_ch", "s_sh", "l_n", "f_h", "r_l", "an_ang", "en_eng", "in_ing", "ian_iang",
    "uan_uang", "ui_uei", "un_uen", "iu_iou",
];

// Rime accepts slash keys, nested maps, and combinations of both. Keep the
// original location when editing, and never silently choose between duplicates.
const APPEND_PATHS: &[&[&str]] = &[
    &["speller/algebra/+"],
    &["speller", "algebra/+"],
    &["speller/algebra", "+"],
    &["speller", "algebra", "+"],
];

fn at_path<'a>(mapping: &'a Mapping, path: &[&str]) -> Option<&'a Value> {
    let (head, tail) = path.split_first()?;
    let value = mapping.get(yaml_str(head))?;
    if tail.is_empty() {
        Some(value)
    } else {
        at_path(value.as_mapping()?, tail)
    }
}

fn replace_at_path(mapping: &mut Mapping, path: &[&str], value: Value) {
    if let Some((head, tail)) = path.split_first() {
        if tail.is_empty() {
            mapping.insert(yaml_str(head), value);
        } else if let Some(Value::Mapping(child)) = mapping.get_mut(yaml_str(head)) {
            replace_at_path(child, tail, value);
        }
    }
}

fn rule_matches_pair(rule: &str, pair: &str) -> bool {
    rules_for_fuzzy_pairs(&[pair.to_string()]).contains(&rule)
        || match rule {
            "derive/^([zcs])h/$1/" | "derive/^([zcs])([^h])/$1h$2/" => {
                matches!(pair, "z_zh" | "c_ch" | "s_sh")
            }
            "derive/^z/zh/" => pair == "z_zh",
            "derive/^c/ch/" => pair == "c_ch",
            "derive/^s/sh/" => pair == "s_sh",
            _ => false,
        }
}

fn pairs_in_rules<'a>(rules: impl Iterator<Item = &'a Value>) -> Vec<String> {
    let rules: Vec<_> = rules.filter_map(Value::as_str).collect();
    FUZZY_PAIRS
        .iter()
        .filter(|pair| rules.iter().any(|rule| rule_matches_pair(rule, pair)))
        .map(|pair| (*pair).to_string())
        .collect()
}

fn pairs_in_patch(patch: &Mapping) -> Vec<String> {
    pairs_in_rules(APPEND_PATHS.iter().flat_map(|path| {
        at_path(patch, path)
            .and_then(Value::as_sequence)
            .into_iter()
            .flatten()
    }))
}

pub(crate) fn detect_fuzzy_pinyin_pairs(contents: &str) -> Vec<String> {
    parse_yaml_mapping(contents)
        .ok()
        .and_then(|root| {
            root.get(yaml_str("patch"))
                .and_then(Value::as_mapping)
                .map(pairs_in_patch)
        })
        .unwrap_or_default()
}

pub(crate) fn merge_fuzzy_pinyin_rules(
    patch: &mut Mapping,
    settings: &RimeIceSettings,
) -> Result<(), RimeError> {
    let rules = if settings.fuzzy_pinyin {
        match settings.fuzzy_pairs.as_deref() {
            Some(pairs) if !pairs.is_empty() => {
                if pairs
                    .iter()
                    .any(|pair| !FUZZY_PAIRS.contains(&pair.as_str()))
                {
                    return Err(RimeError::YamlParseError(
                        "存在无法识别的模糊音选项，已中止写入以免覆盖自定义拼写规则".into(),
                    ));
                }
                rules_for_fuzzy_pairs(pairs)
            }
            // Keep compatibility with clients predating individual pair selection.
            _ => fuzzy_pinyin_algebra_rules(),
        }
    } else {
        Vec::new()
    };
    let desired: Vec<_> = rules.into_iter().map(yaml_str).collect();
    let current_pairs = pairs_in_patch(patch);
    let desired_pairs = pairs_in_rules(desired.iter());
    if current_pairs == desired_pairs {
        // An unrelated settings save must not normalize one-way rules, reorder
        // algebra, or rewrite an advanced/reference-based configuration.
        return Ok(());
    }

    let paths: Vec<_> = APPEND_PATHS
        .iter()
        .filter(|path| at_path(patch, path).is_some())
        .collect();
    if paths.len() > 1 {
        return Err(RimeError::YamlParseError(
            "speller/algebra/+ 存在多处定义，已中止写入以免覆盖自定义拼写规则".into(),
        ));
    }
    let path = paths.first().copied().copied().unwrap_or(APPEND_PATHS[0]);
    let existing = at_path(patch, path);
    let removes_rule = |rule: &str| {
        FUZZY_PAIRS.iter().any(|pair| {
            rule_matches_pair(rule, pair) && !desired_pairs.iter().any(|selected| selected == pair)
        })
    };
    let mut kept = match existing {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Sequence(items)) => items
            .iter()
            .filter(|item| {
                // Retained pairs stay in place, including one-way rules.
                // Algebra is order-sensitive even around custom entries.
                !item.as_str().is_some_and(removes_rule)
            })
            .cloned()
            .collect(),
        _ => {
            return Err(RimeError::YamlParseError(
                "无法合并 speller/algebra/+ 的配置格式，已中止写入以免覆盖自定义拼写规则".into(),
            ))
        }
    };
    let retained_pairs = pairs_in_rules(kept.iter());
    // A removed legacy group may supply one direction while a retained
    // individual rule supplies the other. Pair presence alone would lose the
    // group's still-selected direction, so regenerate those affected pairs too.
    let loses_shared_rule = |pair: &str| {
        existing.and_then(Value::as_sequence).is_some_and(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .any(|rule| rule_matches_pair(rule, pair) && removes_rule(rule))
        })
    };
    let missing_pairs: Vec<_> = desired_pairs
        .iter()
        .filter(|pair| !retained_pairs.contains(pair) || loses_shared_rule(pair))
        .cloned()
        .collect();
    let additions = if current_pairs.is_empty() {
        desired
    } else {
        rules_for_fuzzy_pairs(&missing_pairs)
            .into_iter()
            .map(yaml_str)
            .collect()
    };
    for item in additions {
        if !kept.contains(&item) {
            kept.push(item);
        }
    }
    // Retain an empty append at nested locations instead of leaving an empty
    // parent map that could replace inherited speller/algebra configuration.
    replace_at_path(patch, path, Value::Sequence(kept));
    Ok(())
}

pub(crate) fn rules_for_fuzzy_pairs(pairs: &[String]) -> Vec<&'static str> {
    let mut rules = Vec::new();
    for pair in pairs {
        match pair.as_str() {
            "z_zh" => {
                rules.push("derive/^z([^h])/zh$1/");
                rules.push("derive/^zh/z/");
            }
            "c_ch" => {
                rules.push("derive/^c([^h])/ch$1/");
                rules.push("derive/^ch/c/");
            }
            "s_sh" => {
                rules.push("derive/^s([^h])/sh$1/");
                rules.push("derive/^sh/s/");
            }
            "l_n" => {
                rules.push("derive/^l/n/");
                rules.push("derive/^n/l/");
            }
            "f_h" => {
                rules.push("derive/^f/h/");
                rules.push("derive/^h/f/");
            }
            "r_l" => {
                rules.push("derive/^r/l/");
                rules.push("derive/^l/r/");
            }
            "an_ang" => {
                rules.push("derive/ang$/an/");
                rules.push("derive/an$/ang/");
            }
            "en_eng" => {
                rules.push("derive/eng$/en/");
                rules.push("derive/en$/eng/");
            }
            "in_ing" => {
                rules.push("derive/in$/ing/");
                rules.push("derive/ing$/in/");
            }
            "ian_iang" => {
                rules.push("derive/ian$/iang/");
                rules.push("derive/iang$/ian/");
            }
            "uan_uang" => {
                rules.push("derive/uan$/uang/");
                rules.push("derive/uang$/uan/");
            }
            "ui_uei" => {
                rules.push("derive/ui$/uei/");
            }
            "un_uen" => {
                rules.push("derive/un$/uen/");
            }
            "iu_iou" => {
                rules.push("derive/iu$/iou/");
            }
            _ => {}
        }
    }
    rules.dedup();
    rules
}

pub(crate) fn fuzzy_pinyin_algebra_rules() -> Vec<&'static str> {
    vec![
        "derive/^([zcs])h/$1/",
        "derive/^([zcs])([^h])/$1h$2/",
        "derive/^l/n/",
        "derive/^n/l/",
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
        "derive/un$/uen/",
        "derive/ui$/uei/",
        "derive/iu$/iou/",
    ]
}
