use ::url::Url;

/// Validates and normalizes a base URL for LLM providers.
///
/// # Rules:
/// - HTTPS is always required
/// - HTTP is only allowed for localhost/127.0.0.1/[::1]
/// - Embedded credentials (user:pass@) are forbidden
/// - URL fragments are forbidden
/// - Trailing slash is normalized (added if missing)
///
/// # Returns:
/// - `Ok(String)` with the normalized URL (with trailing slash)
/// - `Err(String)` with a human-readable error message
pub fn validate_base_url(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("URL cannot be empty".into());
    }

    let u = Url::parse(s).map_err(|e| format!("Invalid URL: {}", e))?;

    match u.scheme() {
        "https" => {}
        "http" => {
            let host = u.host_str().unwrap_or("");
            if host != "localhost" && host != "127.0.0.1" && host != "[::1]" {
                return Err("http:// is only allowed for localhost, 127.0.0.1, or [::1]".into());
            }
        }
        other => return Err(format!("unsupported scheme: {}", other)),
    }

    if !u.username().is_empty() || u.password().is_some() {
        return Err("URL must not embed credentials".into());
    }

    if u.fragment().is_some() {
        return Err("URL must not have a fragment".into());
    }

    let mut normalized = u.to_string();
    if !normalized.ends_with('/') {
        normalized.push('/');
    }

    // Re-parse to ensure it's still valid after normalization
    Url::parse(&normalized)
        .map_err(|e| format!("Failed to normalize URL: {}", e))?;

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_anthropic_with_slash() {
        let result = validate_base_url("https://api.anthropic.com/");
        assert_eq!(result.unwrap(), "https://api.anthropic.com/");
    }

    #[test]
    fn accepts_https_anthropic_without_slash() {
        let result = validate_base_url("https://api.anthropic.com");
        assert_eq!(result.unwrap(), "https://api.anthropic.com/");
    }

    #[test]
    fn accepts_http_localhost() {
        let result = validate_base_url("http://localhost:8080/");
        assert_eq!(result.unwrap(), "http://localhost:8080/");
    }

    #[test]
    fn accepts_http_localhost_without_slash() {
        let result = validate_base_url("http://localhost:8080");
        assert_eq!(result.unwrap(), "http://localhost:8080/");
    }

    #[test]
    fn accepts_http_127_0_0_1() {
        let result = validate_base_url("http://127.0.0.1:1234/");
        assert_eq!(result.unwrap(), "http://127.0.0.1:1234/");
    }

    #[test]
    fn accepts_http_ipv6_loopback() {
        let result = validate_base_url("http://[::1]:8080/");
        assert_eq!(result.unwrap(), "http://[::1]:8080/");
    }

    #[test]
    fn rejects_http_example_com() {
        let result = validate_base_url("http://example.com/");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    #[test]
    fn rejects_http_non_loopback_without_slash() {
        let result = validate_base_url("http://api.example.com");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_embedded_credentials() {
        let result = validate_base_url("https://user:pass@example.com/");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("credentials"));
    }

    #[test]
    fn rejects_ftp_scheme() {
        let result = validate_base_url("ftp://example.com/");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported scheme"));
    }

    #[test]
    fn rejects_file_scheme() {
        let result = validate_base_url("file:///etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported scheme"));
    }

    #[test]
    fn rejects_fragment() {
        let result = validate_base_url("https://example.com/#fragment");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("fragment"));
    }

    #[test]
    fn rejects_empty_string() {
        let result = validate_base_url("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn rejects_not_a_url() {
        let result = validate_base_url("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn normalizes_trailing_slash() {
        let result = validate_base_url("https://api.anthropic.com");
        assert_eq!(result.unwrap(), "https://api.anthropic.com/");
    }
}
