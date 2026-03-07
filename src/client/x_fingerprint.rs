// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! X-Fingerprint header management
//!
//! Discord generates a server-side fingerprint via the /experiments endpoint.
//! This fingerprint is expected on certain auth-related endpoints (login, register).
//! Missing it is a detection signal on those specific endpoints.

use crate::{DiscordError, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache duration for the server fingerprint
const FINGERPRINT_TTL: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Fetches and caches Discord's server-generated fingerprint
#[derive(Clone)]
pub struct XFingerprintManager {
    inner: Arc<RwLock<FingerprintCache>>,
}

struct FingerprintCache {
    fingerprint: Option<String>,
    fetched_at: Option<Instant>,
}

impl XFingerprintManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(FingerprintCache {
                fingerprint: None,
                fetched_at: None,
            })),
        }
    }

    /// Get the cached fingerprint, or None if expired/not fetched
    pub async fn get_cached(&self) -> Option<String> {
        let cache = self.inner.read().await;
        if let (Some(ref fp), Some(fetched_at)) = (&cache.fingerprint, cache.fetched_at) {
            if fetched_at.elapsed() < FINGERPRINT_TTL {
                return Some(fp.clone());
            }
        }
        None
    }

    /// Store a freshly fetched fingerprint
    pub async fn store(&self, fingerprint: String) {
        let mut cache = self.inner.write().await;
        cache.fingerprint = Some(fingerprint);
        cache.fetched_at = Some(Instant::now());
    }

    /// Fetch the fingerprint from Discord's experiments endpoint.
    ///
    /// Makes an unauthenticated GET to /api/v10/experiments and extracts
    /// the fingerprint from the response.
    pub async fn fetch_from_discord(&self) -> Result<String> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        let response = client
            .get("https://discord.com/api/v10/experiments")
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://discord.com/login")
            .header("Origin", "https://discord.com")
            .send()
            .await
            .map_err(|e| DiscordError::Http(format!("Failed to fetch fingerprint: {}", e)))?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            let json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| DiscordError::Http(format!("Invalid fingerprint response: {}", e)))?;

            if let Some(fp) = json.get("fingerprint").and_then(|v| v.as_str()) {
                let fp_str = fp.to_string();
                self.store(fp_str.clone()).await;
                tracing::debug!("Fetched X-Fingerprint: {}", &fp_str[..fp_str.len().min(20)]);
                return Ok(fp_str);
            }
        }

        Err(DiscordError::Http(
            "Could not extract fingerprint from experiments endpoint".to_string(),
        ))
    }

    /// Get the fingerprint, fetching if needed
    pub async fn get_or_fetch(&self) -> Result<String> {
        if let Some(cached) = self.get_cached().await {
            return Ok(cached);
        }
        self.fetch_from_discord().await
    }
}

impl Default for XFingerprintManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_empty_initially() {
        let mgr = XFingerprintManager::new();
        assert!(mgr.get_cached().await.is_none());
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let mgr = XFingerprintManager::new();
        mgr.store("fp_abc123".to_string()).await;
        assert_eq!(mgr.get_cached().await, Some("fp_abc123".to_string()));
    }
}
