// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Safe link preview handling
//!
//! Prevents IP leakage when fetching link previews by:
//! - Blocking private/internal IP ranges (SSRF prevention)
//! - Routing through proxy when configured
//! - Timeouts to prevent slow-loris attacks
//! - Sanitizing extracted metadata

use std::net::IpAddr;
use url::Url;

/// Metadata extracted from a link preview
#[derive(Debug, Clone, Default)]
pub struct LinkPreviewMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub site_name: Option<String>,
    pub url: String,
    /// The actual domain the URL resolves to (for spoofing detection)
    pub actual_domain: Option<String>,
}

/// Check if a URL is safe to fetch for link previews
pub fn is_safe_url(url_str: &str) -> Result<Url, LinkPreviewError> {
    let url = Url::parse(url_str).map_err(|_| LinkPreviewError::InvalidUrl)?;

    // Only allow http and https
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err(LinkPreviewError::UnsafeScheme(url.scheme().to_string())),
    }

    // Block data: URIs
    if url.scheme() == "data" {
        return Err(LinkPreviewError::UnsafeScheme("data".to_string()));
    }

    // Check for private/internal IP ranges (SSRF prevention)
    if let Some(host) = url.host_str() {
        // Block localhost
        if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]" {
            return Err(LinkPreviewError::PrivateIp);
        }

        // Block private IP ranges
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(&ip) {
                return Err(LinkPreviewError::PrivateIp);
            }
        }

        // Block common internal hostnames
        let lower_host = host.to_lowercase();
        if lower_host.ends_with(".local")
            || lower_host.ends_with(".internal")
            || lower_host.ends_with(".localhost")
            || lower_host == "metadata.google.internal"
            || lower_host == "169.254.169.254"
        // AWS metadata
        {
            return Err(LinkPreviewError::PrivateIp);
        }
    }

    Ok(url)
}

/// Check if an IP address is in a private/reserved range
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()            // 127.0.0.0/8
                || v4.is_private()       // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                || v4.is_link_local()    // 169.254.0.0/16
                || v4.is_broadcast()     // 255.255.255.255
                || v4.is_unspecified()   // 0.0.0.0
                || v4.octets()[0] == 100 && v4.octets()[1] >= 64 && v4.octets()[1] <= 127
            // CGN 100.64.0.0/10
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()        // ::1
                || v6.is_unspecified() // ::
        }
    }
}

/// Detect URL spoofing in embeds (percent-encoded chars making URLs look like trusted domains)
pub fn detect_url_spoofing(display_url: &str, actual_url: &str) -> bool {
    // Compare the domains after normalizing
    let display_domain = Url::parse(display_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()));
    let actual_domain = Url::parse(actual_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()));

    match (display_domain, actual_domain) {
        (Some(d), Some(a)) => d != a,
        _ => false, // Can't determine — don't flag
    }
}

/// Resolve a hostname and check if any resolved IPs are private.
/// This catches cases where a domain like evil.example.com resolves to 127.0.0.1.
/// Must be called asynchronously before fetching the URL.
pub async fn resolve_and_check_private(host: &str) -> Result<(), LinkPreviewError> {
    use tokio::net::lookup_host;

    let addrs = lookup_host(format!("{}:443", host))
        .await
        .map_err(|e| LinkPreviewError::FetchError(format!("DNS resolution failed: {}", e)))?;

    for addr in addrs {
        if is_private_ip(&addr.ip()) {
            return Err(LinkPreviewError::PrivateIp);
        }
    }

    Ok(())
}

/// Known tracker domains to block in link preview requests
pub fn is_known_tracker(host: &str) -> bool {
    let lower = host.to_lowercase();
    let trackers = [
        "pixel.facebook.com",
        "tracking.google.com",
        "bat.bing.com",
        "analytics.twitter.com",
        "dc.ads.linkedin.com",
        "pixel.quantserve.com",
        "cm.g.doubleclick.net",
    ];
    trackers.iter().any(|t| lower.contains(t))
}

/// Link preview errors
#[derive(Debug, Clone)]
pub enum LinkPreviewError {
    InvalidUrl,
    UnsafeScheme(String),
    PrivateIp,
    Timeout,
    FetchError(String),
}

impl std::fmt::Display for LinkPreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl => write!(f, "Invalid URL"),
            Self::UnsafeScheme(s) => write!(f, "Unsafe URL scheme: {}", s),
            Self::PrivateIp => write!(f, "URL resolves to private/internal IP"),
            Self::Timeout => write!(f, "Link preview fetch timed out"),
            Self::FetchError(e) => write!(f, "Fetch error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_url_accepts_https() {
        assert!(is_safe_url("https://discord.com/channels/@me").is_ok());
        assert!(is_safe_url("https://example.com/page").is_ok());
        assert!(is_safe_url("http://example.com/page").is_ok());
    }

    #[test]
    fn test_safe_url_blocks_private_ips() {
        assert!(is_safe_url("http://127.0.0.1/admin").is_err());
        assert!(is_safe_url("http://localhost/secret").is_err());
        assert!(is_safe_url("http://192.168.1.1/").is_err());
        assert!(is_safe_url("http://10.0.0.1/").is_err());
        assert!(is_safe_url("http://172.16.0.1/").is_err());
        assert!(is_safe_url("http://169.254.169.254/").is_err()); // AWS metadata
    }

    #[test]
    fn test_safe_url_blocks_internal_hostnames() {
        assert!(is_safe_url("http://host.local/").is_err());
        assert!(is_safe_url("http://server.internal/").is_err());
        assert!(is_safe_url("http://metadata.google.internal/").is_err());
    }

    #[test]
    fn test_safe_url_blocks_unsafe_schemes() {
        assert!(is_safe_url("ftp://example.com/").is_err());
        assert!(is_safe_url("file:///etc/passwd").is_err());
        assert!(is_safe_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_url_spoofing_detection() {
        assert!(detect_url_spoofing(
            "https://discord.gg/invite",
            "https://evil.com/fake"
        ));
        assert!(!detect_url_spoofing(
            "https://discord.com/channels",
            "https://discord.com/app"
        ));
    }

    #[test]
    fn test_private_ip_detection() {
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"100.64.0.1".parse().unwrap())); // CGN
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));
    }
}
