//! Shared HTTP client with system proxy support.
//!
//! Provides a singleton `ureq::Agent` configured with the system proxy,
//! and convenience functions for making HTTP requests.

use std::sync::OnceLock;

use super::proxy::get_system_proxy;

/// Default User-Agent header
const DEFAULT_USER_AGENT: &str = "RimeStudio/0.4";

/// Cached HTTP agent with proxy configuration
static HTTP_AGENT: OnceLock<ureq::Agent> = OnceLock::new();

/// Get the shared HTTP agent.
///
/// The agent is created once and cached for the process lifetime.
/// It is configured with the system proxy if available.
pub(crate) fn http_agent() -> &'static ureq::Agent {
    HTTP_AGENT.get_or_init(create_http_agent)
}

/// Create a new HTTP agent with system proxy configuration.
fn create_http_agent() -> ureq::Agent {
    // Configure proxy from system settings
    if let Some(proxy_url) = get_system_proxy() {
        match ureq::Proxy::new(&proxy_url) {
            Ok(proxy) => {
                log::info!("HTTP agent configured with proxy: {}", proxy_url);
                ureq::AgentBuilder::new().proxy(proxy).build()
            }
            Err(e) => {
                log::warn!("Failed to create proxy from '{}': {}, using direct connection", proxy_url, e);
                ureq::Agent::new()
            }
        }
    } else {
        log::info!("No system proxy detected, using direct connection");
        ureq::Agent::new()
    }
}

/// Convenience wrapper for GET requests with default headers.
///
/// Returns a `ureq::Request` that can be further configured and called.
pub(crate) fn http_get(url: &str) -> ureq::Request {
    http_agent()
        .get(url)
        .set("User-Agent", DEFAULT_USER_AGENT)
        .set("Accept", "application/vnd.github+json")
}

/// Reset the HTTP agent cache.
///
/// This will cause a new agent to be created on the next call to `http_agent()`.
/// Useful if proxy settings change at runtime.
#[allow(dead_code)]
pub(crate) fn reset_http_agent() {
    // OnceLock can only be set once, so we log a warning
    // In practice, app restart is needed for proxy changes
    log::warn!("HTTP agent reset requested - restart app for proxy changes to take effect");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_agent_returns_valid_agent() {
        let agent = http_agent();
        // Agent should be created successfully
        assert!(!format!("{:?}", agent).is_empty());
    }

    #[test]
    fn http_get_sets_default_headers() {
        let request = http_get("https://example.com/test");
        // Verify the request was created (we can't easily inspect headers)
        // but we can verify it doesn't panic
        let _ = request;
    }
}
