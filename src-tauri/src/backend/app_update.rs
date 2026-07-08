const APP_RELEASE_API_URL: &str = "https://api.github.com/repos/l1280776919/rime-studio/releases/latest";
const APP_RELEASES_URL: &str = "https://github.com/l1280776919/rime-studio/releases";

#[derive(Debug, Serialize)]
struct AppUpdateInfo {
    current_version: String,
    latest_version: Option<String>,
    release_name: Option<String>,
    release_notes: Option<String>,
    published_at: Option<String>,
    release_url: String,
    asset_name: Option<String>,
    asset_url: Option<String>,
    update_available: bool,
}

fn normalize_version(value: &str) -> &str {
    value.trim().trim_start_matches('v').trim_start_matches('V')
}

fn parse_semver(value: &str) -> Option<(u64, u64, u64, Vec<String>)> {
    let normalized = normalize_version(value);
    let (core, suffix) = normalized.split_once('-').unwrap_or((normalized, ""));
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let suffix_parts = if suffix.is_empty() {
        Vec::new()
    } else {
        suffix.split('.').map(str::to_string).collect()
    };
    Some((major, minor, patch, suffix_parts))
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let Some((latest_major, latest_minor, latest_patch, latest_suffix)) = parse_semver(latest) else {
        return false;
    };
    let Some((current_major, current_minor, current_patch, current_suffix)) = parse_semver(current) else {
        return false;
    };

    (latest_major, latest_minor, latest_patch) > (current_major, current_minor, current_patch)
        || ((latest_major, latest_minor, latest_patch) == (current_major, current_minor, current_patch)
            && !current_suffix.is_empty()
            && latest_suffix.is_empty())
}

fn release_asset_score(name: &str) -> i32 {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".exe") {
        30
    } else if lower.ends_with(".msi") {
        20
    } else {
        0
    }
}

fn check_app_update_sync() -> Result<AppUpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let response = ureq::get(APP_RELEASE_API_URL)
        .set("User-Agent", "RimeStudio/0.2")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|err| format!("获取 Rime Studio 发布信息失败: {err}"))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|err| format!("解析 Rime Studio 发布信息失败: {err}"))?;

    let latest_version = json["tag_name"]
        .as_str()
        .or_else(|| json["name"].as_str())
        .map(str::to_string);
    let release_url = json["html_url"]
        .as_str()
        .unwrap_or(APP_RELEASES_URL)
        .to_string();
    let release_name = json["name"].as_str().map(str::to_string);
    let release_notes = json["body"].as_str().map(str::to_string);
    let published_at = json["published_at"].as_str().map(str::to_string);
    let update_available = latest_version
        .as_deref()
        .map(|latest| version_is_newer(latest, &current_version))
        .unwrap_or(false);

    let selected_asset = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .filter_map(|asset| {
                    let name = asset["name"].as_str()?;
                    let score = release_asset_score(name);
                    if score == 0 {
                        return None;
                    }
                    Some((
                        score,
                        name.to_string(),
                        asset["browser_download_url"].as_str()?.to_string(),
                    ))
                })
                .max_by_key(|(score, name, _)| (*score, name.contains("setup") || name.contains("install")))
        });

    let (asset_name, asset_url) = selected_asset
        .map(|(_, name, url)| (Some(name), Some(url)))
        .unwrap_or((None, None));

    Ok(AppUpdateInfo {
        current_version,
        latest_version,
        release_name,
        release_notes,
        published_at,
        release_url,
        asset_name,
        asset_url,
        update_available,
    })
}

#[cfg(test)]
mod app_update_tests {
    use super::version_is_newer;

    #[test]
    fn compares_release_versions() {
        assert!(version_is_newer("v0.2.11", "0.2.10"));
        assert!(version_is_newer("1.0.0", "0.9.9"));
        assert!(!version_is_newer("v0.2.10", "0.2.10"));
        assert!(!version_is_newer("0.2.9", "0.2.10"));
    }

    #[test]
    fn treats_stable_release_as_newer_than_current_prerelease() {
        assert!(version_is_newer("1.0.0", "1.0.0-beta.1"));
        assert!(!version_is_newer("1.0.0-beta.1", "1.0.0"));
    }
}
