use crate::*;

pub(crate) fn font_display_name(value_name: &str) -> String {
    value_name
        .rsplit_once(" (")
        .map(|(name, _)| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| value_name.trim().to_string())
}

#[cfg(windows)]
pub(crate) fn list_system_fonts_sync() -> Result<Vec<String>, RimeError> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    let mut names = std::collections::BTreeSet::new();
    let hives = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
        ),
    ];

    for (hive, path) in hives {
        let Ok(key) = RegKey::predef(hive).open_subkey_with_flags(path, KEY_READ) else {
            continue;
        };
        for (name, _) in key.enum_values().flatten() {
            let display = font_display_name(&name);
            if !display.is_empty() {
                names.insert(display);
            }
        }
    }

    Ok(names.into_iter().collect())
}

#[cfg(not(windows))]
pub(crate) fn list_system_fonts_sync() -> Result<Vec<String>, RimeError> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_truetype_suffix_from_font_registry_name() {
        assert_eq!(
            font_display_name("Microsoft YaHei UI (TrueType)"),
            "Microsoft YaHei UI"
        );
        assert_eq!(font_display_name("Sarasa Gothic"), "Sarasa Gothic");
    }
}
