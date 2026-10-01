use crate::backend::*;
use crate::*;

const APP_RELEASE_API_URL: &str =
    "https://api.github.com/repos/l1280776919/rime-studio/releases/latest";
const APP_RELEASES_URL: &str = "https://github.com/l1280776919/rime-studio/releases";

pub(crate) fn normalize_version(value: &str) -> &str {
    value.trim().trim_start_matches('v').trim_start_matches('V')
}

pub(crate) fn version_is_newer(latest: &str, current: &str) -> bool {
    match (
        semver::Version::parse(normalize_version(latest)),
        semver::Version::parse(normalize_version(current)),
    ) {
        (Ok(latest), Ok(current)) => latest.cmp_precedence(&current).is_gt(),
        _ => false,
    }
}

fn matches_installer_architecture(name: &str, arch: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let arm = name.contains("arm64") || name.contains("aarch64");
    let x64 = name.contains("x64") || name.contains("x86_64") || name.contains("amd64");
    let x86 = !x64 && (name.contains("x86") || name.contains("i686") || name.contains("win32"));
    match arch {
        "aarch64" => !x64 && !x86,
        "x86_64" => !arm && !x86,
        "x86" => !arm && !x64,
        _ => !arm && !x64 && !x86,
    }
}

pub(crate) fn release_asset_score(name: &str) -> i32 {
    // Automatic launch currently supports EXE; unsupported formats remain available on the release page.
    if is_installer_filename(name) && matches_installer_architecture(name, std::env::consts::ARCH) {
        30
    } else {
        0
    }
}

pub(crate) fn check_app_update_sync() -> Result<AppUpdateInfo, RimeError> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let response = http_get(APP_RELEASE_API_URL)
        .call()
        .map_err(|err| RimeError::NetworkError(format!("获取 Rime Studio 发布信息失败: {err}")))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|err| RimeError::NetworkError(format!("解析 Rime Studio 发布信息失败: {err}")))?;

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

    let selected_asset = json["assets"].as_array().and_then(|assets| {
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
                    asset["size"].as_u64(),
                ))
            })
            .max_by_key(|(score, name, _, _)| {
                (*score, {
                    let name = name.to_ascii_lowercase();
                    name.contains("setup") || name.contains("install")
                })
            })
    });

    let (asset_name, asset_url, asset_size) = selected_asset
        .map(|(_, name, url, size)| (Some(name), Some(url), size))
        .unwrap_or((None, None, None));

    Ok(AppUpdateInfo {
        current_version,
        latest_version,
        release_name,
        release_notes,
        published_at,
        release_url,
        asset_name,
        asset_url,
        asset_size,
        update_available,
    })
}

pub(crate) fn download_app_update_sync() -> Result<RimeDownloadResult, RimeError> {
    let info = check_app_update_sync()?;
    let download_url = info
        .asset_url
        .ok_or_else(|| RimeError::DownloadError("未找到可下载的安装包".to_string()))?;
    let filename = info
        .asset_name
        .unwrap_or_else(|| "RimeStudio-Installer.exe".to_string());

    download_installer_asset(&filename, &download_url, info.asset_size, None)
}

#[cfg(test)]
mod app_update_tests {
    use super::version_is_newer;

    #[test]
    fn compares_prerelease_identifiers_and_ignores_build_metadata() {
        assert!(super::version_is_newer("1.0.0-rc.10", "1.0.0-rc.9"));
        assert!(super::version_is_newer("1.0.0-beta", "1.0.0-alpha.9"));
        assert!(super::version_is_newer("1.0.1+build.2", "1.0.0+build.9"));
        assert!(!super::version_is_newer("1.0.0+build.2", "1.0.0+build.1"));
        for invalid in ["1.0.0.2", "1.0.0-01", "01.0.0", "1.0.0-", ""] {
            assert!(!super::version_is_newer(invalid, "1.0.0"));
        }
    }

    #[test]
    fn selects_supported_installers_for_the_current_architecture() {
        assert!(super::release_asset_score("RimeStudio_SETUP.EXE") > 0);
        assert_eq!(super::release_asset_score("../setup.exe"), 0);
        assert_eq!(super::release_asset_score("setup.msi"), 0);
        assert!(!super::matches_installer_architecture(
            "setup-arm64.exe",
            "x86_64"
        ));
        assert!(!super::matches_installer_architecture(
            "setup-x64.exe",
            "aarch64"
        ));
        assert!(super::matches_installer_architecture(
            "setup-arm64.exe",
            "aarch64"
        ));
    }

    #[test]
    pub(crate) fn compares_release_versions() {
        assert!(version_is_newer("v0.2.11", "0.2.10"));
        assert!(version_is_newer("1.0.0", "0.9.9"));
        assert!(!version_is_newer("v0.2.10", "0.2.10"));
        assert!(!version_is_newer("0.2.9", "0.2.10"));
    }

    #[test]
    pub(crate) fn treats_stable_release_as_newer_than_current_prerelease() {
        assert!(version_is_newer("1.0.0", "1.0.0-beta.1"));
        assert!(!version_is_newer("1.0.0-beta.1", "1.0.0"));
    }
}
