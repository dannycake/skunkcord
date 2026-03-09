// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Content sanitization and safety
//!
//! Sanitizes message content before rendering to prevent XSS and other attacks.
//!
//! # Example
//!
//! ```
//! use skunkcord::security::content::{strip_tracking_params, sanitize_for_display};
//!
//! let url = "https://example.com/page?utm_source=twitter&id=123";
//! let clean = strip_tracking_params(url);
//! assert!(clean.contains("id=123"));
//! assert!(!clean.contains("utm_source"));
//!
//! let safe = sanitize_for_display("<script>alert('xss')</script>");
//! assert!(!safe.contains("<script>"));
//! ```

/// Tracking URL parameters to strip (ClearURLs feature)
pub const TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_name",
    "utm_cid",
    "utm_reader",
    "utm_viz_id",
    "utm_pubreferrer",
    "utm_swu",
    "fbclid",
    "gclid",
    "gclsrc",
    "dclid",
    "mc_eid",
    "mc_cid",
    "igshid",
    "si",      // YouTube, Spotify
    "feature", // YouTube
    "ref",
    "ref_src",
    "ref_url",
    "s", // Twitter/X share param
    "t", // Reddit share param
    "share_id",
    "context", // various
    "_hsenc",
    "_hsmi",
    "mkt_tok",
    "trk",
    "trkCampaign",
    "sc_campaign",
    "sc_channel",
    "sc_content",
    "sc_medium",
    "sc_outcome",
    "sc_geo",
    "sc_country",
];

/// Strip tracking parameters from a URL
pub fn strip_tracking_params(url_str: &str) -> String {
    match url::Url::parse(url_str) {
        Ok(mut url) => {
            let pairs: Vec<(String, String)> = url
                .query_pairs()
                .filter(|(key, _)| {
                    let key_lower = key.to_lowercase();
                    !TRACKING_PARAMS.iter().any(|tp| key_lower == *tp)
                })
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if pairs.is_empty() {
                url.set_query(None);
            } else {
                let new_query: String = pairs
                    .iter()
                    .map(|(k, v)| {
                        if v.is_empty() {
                            k.clone()
                        } else {
                            format!("{}={}", k, v)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("&");
                url.set_query(Some(&new_query));
            }

            url.to_string()
        }
        Err(_) => url_str.to_string(), // Return original if parsing fails
    }
}

/// Strip tracking params from all URLs in a message
pub fn clean_message_urls(content: &str) -> String {
    // Simple URL detection — matches http(s):// followed by non-whitespace
    let url_re = regex::Regex::new(r"https?://\S+").unwrap();

    url_re
        .replace_all(content, |caps: &regex::Captures| {
            strip_tracking_params(&caps[0])
        })
        .to_string()
}

/// Sanitize content for safe display — escape HTML entities
pub fn sanitize_for_display(content: &str) -> String {
    content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Check if a URL is from Discord's CDN (safe for direct loading)
pub fn is_discord_cdn_url(url: &str) -> bool {
    let safe_domains = [
        "cdn.discordapp.com",
        "media.discordapp.net",
        "images-ext-1.discordapp.net",
        "images-ext-2.discordapp.net",
        "discord.com",
    ];

    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return safe_domains
                .iter()
                .any(|d| host == *d || host.ends_with(&format!(".{}", d)));
        }
    }
    false
}

/// Check if content contains data: URIs (potential XSS vector)
pub fn contains_data_uri(content: &str) -> bool {
    content.contains("data:") && (content.contains("base64") || content.contains("text/html"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_tracking_params() {
        assert_eq!(
            strip_tracking_params("https://example.com/page?utm_source=twitter&id=123"),
            "https://example.com/page?id=123"
        );
        assert_eq!(
            strip_tracking_params("https://example.com/page?fbclid=abc123"),
            "https://example.com/page"
        );
        assert_eq!(
            strip_tracking_params("https://example.com/page?id=123"),
            "https://example.com/page?id=123"
        );
    }

    #[test]
    fn test_strip_all_tracking() {
        let url = "https://example.com/?utm_source=a&utm_medium=b&utm_campaign=c";
        assert_eq!(strip_tracking_params(url), "https://example.com/");
    }

    #[test]
    fn test_clean_message_urls() {
        let msg = "Check out https://example.com/cool?utm_source=discord&id=5 it's great!";
        let cleaned = clean_message_urls(msg);
        assert!(cleaned.contains("id=5"));
        assert!(!cleaned.contains("utm_source"));
    }

    #[test]
    fn test_sanitize_for_display() {
        assert_eq!(
            sanitize_for_display("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_discord_cdn_detection() {
        assert!(is_discord_cdn_url(
            "https://cdn.discordapp.com/avatars/123/abc.png"
        ));
        assert!(is_discord_cdn_url(
            "https://media.discordapp.net/attachments/123/456/file.png"
        ));
        assert!(!is_discord_cdn_url("https://evil.com/fake-discord.png"));
    }

    #[test]
    fn test_data_uri_detection() {
        assert!(contains_data_uri(
            "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg=="
        ));
        assert!(!contains_data_uri("https://example.com/image.png"));
        assert!(!contains_data_uri("regular message text"));
    }
}
