//! Redaction helpers for safe logging of sensitive data.

/// Redact a URL to show only scheme, host, and port (drop path, query, fragment).
pub fn url_host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str().map(|h| {
                let scheme = u.scheme();
                let port =
                    u.port().map(|p| format!(":{p}")).unwrap_or_default();
                format!("{scheme}://{h}{port}")
            })
        })
        .unwrap_or_else(|| "<invalid-url>".into())
}

/// Redact an API key to show first 4 chars + length.
#[allow(dead_code)]
pub fn key_prefix(key: &str) -> String {
    let prefix: String = key.chars().take(4).collect();
    format!("{prefix}…(len={})", key.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_host_https() {
        assert_eq!(
            url_host("https://api.anthropic.com/v1/messages?key=secret"),
            "https://api.anthropic.com"
        );
    }

    #[test]
    fn test_url_host_http_localhost() {
        assert_eq!(
            url_host("http://localhost:8080/foo"),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_url_host_invalid() {
        assert_eq!(url_host("not a url"), "<invalid-url>");
    }

    #[test]
    fn test_key_prefix_long() {
        assert_eq!(key_prefix("sk-1234567890abcdef"), "sk-1…(len=19)");
    }

    #[test]
    fn test_key_prefix_short() {
        assert_eq!(key_prefix("ab"), "ab…(len=2)");
    }
}
