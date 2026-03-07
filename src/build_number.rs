// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Dynamic Discord build number fetching
//!
//! Scrapes the current client build number from Discord's web app.
//! The build number changes frequently and using a stale value is a
//! detection signal. This module fetches the real value and caches it.

use crate::{DiscordError, Result};
use regex::Regex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Default (fallback) build number — updated periodically.
/// Only used if scraping fails entirely.
const FALLBACK_BUILD_NUMBER: u64 = 348000;

/// How long to cache the build number before re-fetching
const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours

/// Fetches and caches Discord's current client build number.
#[derive(Clone)]
pub struct BuildNumberFetcher {
    inner: Arc<RwLock<BuildNumberCache>>,
}

struct BuildNumberCache {
    build_number: u64,
    fetched_at: Option<Instant>,
}

impl BuildNumberFetcher {
    /// Create a new fetcher with the fallback build number
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BuildNumberCache {
                build_number: FALLBACK_BUILD_NUMBER,
                fetched_at: None,
            })),
        }
    }

    /// Get the current build number, fetching from Discord if stale/never fetched.
    pub async fn get(&self) -> u64 {
        {
            let cache = self.inner.read().await;
            if let Some(fetched_at) = cache.fetched_at {
                if fetched_at.elapsed() < CACHE_TTL {
                    return cache.build_number;
                }
            }
        }

        // Need to fetch — try to update
        match self.fetch_from_discord().await {
            Ok(num) => {
                let mut cache = self.inner.write().await;
                cache.build_number = num;
                cache.fetched_at = Some(Instant::now());
                tracing::info!("Updated Discord build number to {}", num);
                num
            }
            Err(e) => {
                tracing::warn!("Failed to fetch build number: {}, using cached/fallback", e);
                self.inner.read().await.build_number
            }
        }
    }

    /// Force a refresh of the build number
    pub async fn refresh(&self) -> Result<u64> {
        let num = self.fetch_from_discord().await?;
        let mut cache = self.inner.write().await;
        cache.build_number = num;
        cache.fetched_at = Some(Instant::now());
        Ok(num)
    }

    /// Fetch the build number by scraping Discord's web app.
    ///
    /// Process:
    /// 1. Fetch https://discord.com/app (or /login)
    /// 2. Find the main JS bundle URL from script tags
    /// 3. Fetch the JS bundle
    /// 4. Extract buildNumber from the JS source
    async fn fetch_from_discord(&self) -> Result<u64> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        // Step 1: Fetch the Discord login page
        let html = client
            .get("https://discord.com/login")
            .send()
            .await
            .map_err(|e| DiscordError::Http(format!("Failed to fetch Discord page: {}", e)))?
            .text()
            .await
            .map_err(|e| DiscordError::Http(format!("Failed to read Discord page: {}", e)))?;

        // Step 2: Find JS asset URLs from the HTML
        // Discord loads multiple JS chunks; the build number is typically in
        // a chunk whose filename contains "sentry" or in the main app bundle.
        let script_re = Regex::new(r#"(/assets/[a-f0-9]+\.js)"#)
            .map_err(|e| DiscordError::Http(format!("Regex error: {}", e)))?;

        let mut script_urls: Vec<String> = script_re
            .captures_iter(&html)
            .map(|cap| format!("https://discord.com{}", &cap[1]))
            .collect();

        if script_urls.is_empty() {
            return Err(DiscordError::Http(
                "No JS assets found on Discord page".to_string(),
            ));
        }

        // Step 3: Check each JS bundle for the build number (try last few first,
        // as the build number is typically in the later-loaded chunks)
        script_urls.reverse();

        let build_re = Regex::new(r#"buildNumber['":\s]*[=:]\s*(\d{5,7})"#)
            .map_err(|e| DiscordError::Http(format!("Regex error: {}", e)))?;

        // Also try the "build_number" pattern
        let build_re2 = Regex::new(r#"build_number['":\s]*[=:]\s*(\d{5,7})"#)
            .map_err(|e| DiscordError::Http(format!("Regex error: {}", e)))?;

        // Only check a reasonable number of assets
        for url in script_urls.iter().take(8) {
            let js = match client.get(url).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => text,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            // Try both patterns
            if let Some(caps) = build_re.captures(&js).or_else(|| build_re2.captures(&js)) {
                if let Ok(num) = caps[1].parse::<u64>() {
                    if num > 100000 {
                        // Sanity check: build numbers are > 100k
                        return Ok(num);
                    }
                }
            }
        }

        Err(DiscordError::Http(
            "Build number not found in Discord JS assets".to_string(),
        ))
    }
}

impl Default for BuildNumberFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_build_number() {
        let fetcher = BuildNumberFetcher::new();
        // Without fetching, should return fallback
        let rt = tokio::runtime::Runtime::new().unwrap();
        let num = rt.block_on(async {
            // Read directly from cache without triggering fetch
            fetcher.inner.read().await.build_number
        });
        assert_eq!(num, FALLBACK_BUILD_NUMBER);
    }

    #[test]
    fn test_build_number_regex_patterns() {
        let build_re = Regex::new(r#"buildNumber['":\s]*[=:]\s*(\d{5,7})"#).unwrap();
        let build_re2 = Regex::new(r#"build_number['":\s]*[=:]\s*(\d{5,7})"#).unwrap();

        // Pattern 1: buildNumber:"345123" — the quotes are around the value not the key
        let test1 = r#"buildNumber:345123,other"#;
        let caps = build_re.captures(test1).unwrap();
        assert_eq!(&caps[1], "345123");

        // Pattern 2: buildNumber: 345123
        let test2 = r#"buildNumber: 345123,"#;
        let caps = build_re.captures(test2).unwrap();
        assert_eq!(&caps[1], "345123");

        // Pattern 3: build_number: 345123
        let test3 = r#"build_number: 345123"#;
        let caps = build_re2.captures(test3).unwrap();
        assert_eq!(&caps[1], "345123");

        // Pattern 4: build_number=345123
        let test4 = r#"build_number=345123"#;
        let caps = build_re2.captures(test4).unwrap();
        assert_eq!(&caps[1], "345123");

        // Pattern 5: buildNumber=345123
        let test5 = r#"e.buildNumber=345123"#;
        let caps = build_re.captures(test5).unwrap();
        assert_eq!(&caps[1], "345123");
    }
}
