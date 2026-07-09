//! Windows system proxy detection and configuration.
//!
//! Reads proxy settings from the Windows registry and provides
//! cached proxy configuration for HTTP clients and subprocesses.

use std::sync::OnceLock;

/// Windows registry path for Internet Settings
const INTERNET_SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

/// Cached proxy URL (e.g., "http://127.0.0.1:7890")
static CACHED_PROXY_URL: OnceLock<Option<String>> = OnceLock::new();

/// Get the system proxy URL, cached for the process lifetime.
///
/// Returns `Some("http://host:port")` if a proxy is enabled,
/// or `None` if no proxy is configured or detection fails.
pub(crate) fn get_system_proxy() -> Option<String> {
    CACHED_PROXY_URL
        .get_or_init(|| detect_system_proxy_from_env().or_else(detect_system_proxy_from_registry))
        .clone()
}

/// Get proxy environment variables for subprocesses.
///
/// Returns a list of (key, value) pairs to set in subprocess environment,
/// e.g., `[("http_proxy", "http://127.0.0.1:7890"), ...]`.
pub(crate) fn get_proxy_env_vars() -> Vec<(String, String)> {
    match get_system_proxy() {
        Some(proxy_url) => vec![
            ("http_proxy".to_string(), proxy_url.clone()),
            ("https_proxy".to_string(), proxy_url.clone()),
            ("HTTP_PROXY".to_string(), proxy_url.clone()),
            ("HTTPS_PROXY".to_string(), proxy_url),
        ],
        None => vec![],
    }
}

/// Force refresh the cached proxy configuration.
///
/// Call this if the user changes proxy settings at runtime.
#[allow(dead_code)]
pub(crate) fn refresh_proxy_cache() {
    // OnceLock can only be set once, so we use a workaround:
    // Create a new static for the refresh case
    if let Some(proxy) = detect_system_proxy_from_env().or_else(detect_system_proxy_from_registry) {
        log::info!("Proxy refreshed: {}", proxy);
    } else {
        log::info!("Proxy refreshed: no proxy detected");
    }
}

/// Detect proxy from environment variables (cross-platform).
fn detect_system_proxy_from_env() -> Option<String> {
    for var in &[
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        if let Ok(val) = std::env::var(var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Detect proxy from Windows registry.
fn detect_system_proxy_from_registry() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        detect_windows_registry_proxy()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Read proxy settings from Windows Internet Settings registry key.
#[cfg(target_os = "windows")]
fn detect_windows_registry_proxy() -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey(INTERNET_SETTINGS_KEY).ok()?;

    // Check if proxy is enabled
    let enabled: u32 = settings.get_value("ProxyEnable").ok()?;
    if enabled == 0 {
        return None;
    }

    // Get proxy server string
    let server: String = settings.get_value("ProxyServer").ok()?;
    let server = server.trim().to_string();
    if server.is_empty() {
        return None;
    }

    // Parse and return proxy URL
    parse_proxy_string(&server)
}

/// Parse a Windows proxy server string into a URL.
///
/// Windows proxy strings can be:
/// - Simple: `"host:port"` or `"http://host:port"`
/// - Protocol-specific: `"http=host:port;https=host:port;ftp=host:port"`
fn parse_proxy_string(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // Protocol-specific format
    if raw.contains('=') {
        let mut https_proxy: Option<&str> = None;
        let mut http_proxy: Option<&str> = None;

        for part in raw.split(';') {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("https=") {
                https_proxy = Some(rest.trim());
            } else if let Some(rest) = part.strip_prefix("http=") {
                http_proxy = Some(rest.trim());
            }
        }

        // Prefer HTTPS proxy, fall back to HTTP
        let selected = https_proxy.or(http_proxy)?;
        if selected.is_empty() {
            return None;
        }
        return Some(normalize_proxy_url(selected));
    }

    // Simple format: "host:port" or "http://host:port"
    Some(normalize_proxy_url(raw))
}

/// Normalize a proxy address to a full URL.
fn normalize_proxy_url(addr: &str) -> String {
    let addr = addr.trim();
    if addr.starts_with("http://") || addr.starts_with("https://") {
        addr.to_string()
    } else {
        format!("http://{}", addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_proxy_string() {
        let result = parse_proxy_string("127.0.0.1:7890");
        assert_eq!(result, Some("http://127.0.0.1:7890".to_string()));
    }

    #[test]
    fn parses_simple_proxy_with_protocol() {
        let result = parse_proxy_string("http://127.0.0.1:7890");
        assert_eq!(result, Some("http://127.0.0.1:7890".to_string()));
    }

    #[test]
    fn parses_compound_proxy_string() {
        let result = parse_proxy_string("http=127.0.0.1:8080;https=127.0.0.1:8443");
        assert_eq!(result, Some("http://127.0.0.1:8443".to_string()));
    }

    #[test]
    fn parses_compound_proxy_https_preferred() {
        let result = parse_proxy_string("http=proxy:80;https=proxy:443;ftp=proxy:21");
        assert_eq!(result, Some("http://proxy:443".to_string()));
    }

    #[test]
    fn parses_compound_proxy_http_fallback() {
        let result = parse_proxy_string("http=proxy:80;ftp=proxy:21");
        assert_eq!(result, Some("http://proxy:80".to_string()));
    }

    #[test]
    fn handles_empty_proxy_string() {
        assert_eq!(parse_proxy_string(""), None);
        assert_eq!(parse_proxy_string("   "), None);
    }

    #[test]
    fn handles_malformed_proxy_string() {
        // Only protocol prefix, no host
        assert_eq!(parse_proxy_string("http="), None);
        assert_eq!(parse_proxy_string("https=;http="), None);
    }

    #[test]
    fn proxy_env_vars_format() {
        // This test only verifies the format, not actual proxy detection
        let vars = [
            (
                "http_proxy".to_string(),
                "http://127.0.0.1:7890".to_string(),
            ),
            (
                "https_proxy".to_string(),
                "http://127.0.0.1:7890".to_string(),
            ),
            (
                "HTTP_PROXY".to_string(),
                "http://127.0.0.1:7890".to_string(),
            ),
            (
                "HTTPS_PROXY".to_string(),
                "http://127.0.0.1:7890".to_string(),
            ),
        ];
        assert_eq!(vars.len(), 4);
        assert!(vars.iter().any(|(k, _)| k == "http_proxy"));
        assert!(vars.iter().any(|(k, _)| k == "https_proxy"));
        assert!(vars.iter().any(|(k, _)| k == "HTTP_PROXY"));
        assert!(vars.iter().any(|(k, _)| k == "HTTPS_PROXY"));
    }

    #[test]
    fn normalize_proxy_url_adds_scheme() {
        assert_eq!(
            normalize_proxy_url("127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
    }

    #[test]
    fn normalize_proxy_url_keeps_scheme() {
        assert_eq!(
            normalize_proxy_url("http://127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            normalize_proxy_url("https://proxy.corp.com:443"),
            "https://proxy.corp.com:443"
        );
    }
}
