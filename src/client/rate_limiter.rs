// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Per-route rate limit bucket tracking
//!
//! Discord uses per-route rate limit buckets. Each route has its own
//! limit, and the response headers tell us the current state.
//! This module tracks buckets proactively to avoid hitting 429s.
//!
//! Headers parsed:
//! - X-RateLimit-Limit: total requests allowed per window
//! - X-RateLimit-Remaining: requests remaining in current window
//! - X-RateLimit-Reset: Unix timestamp when the window resets
//! - X-RateLimit-Reset-After: seconds until reset
//! - X-RateLimit-Bucket: opaque bucket identifier
//! - X-RateLimit-Global: whether this is a global rate limit

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Rate limit info for a single bucket
#[derive(Debug, Clone)]
pub struct RateLimitBucket {
    /// Opaque bucket ID from Discord
    pub bucket_id: String,
    /// Total requests allowed per window
    pub limit: u32,
    /// Requests remaining in current window
    pub remaining: u32,
    /// When the current window resets
    pub resets_at: Instant,
    /// Last updated
    pub updated_at: Instant,
}

impl RateLimitBucket {
    /// Check if we can make a request right now
    pub fn can_request(&self) -> bool {
        if Instant::now() >= self.resets_at {
            return true; // Window has reset
        }
        self.remaining > 0
    }

    /// Get the time to wait before we can make a request
    pub fn wait_time(&self) -> Option<Duration> {
        if self.can_request() {
            None
        } else {
            let now = Instant::now();
            if now < self.resets_at {
                Some(self.resets_at - now)
            } else {
                None
            }
        }
    }
}

/// Rate limiter that tracks per-route buckets
#[derive(Debug, Default)]
pub struct RateLimiter {
    /// Route → bucket ID mapping
    route_buckets: HashMap<String, String>,
    /// Bucket ID → bucket info
    buckets: HashMap<String, RateLimitBucket>,
    /// Global rate limit (if active)
    global_reset: Option<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update rate limit state from response headers
    pub fn update_from_headers(&mut self, route: &str, headers: &reqwest::header::HeaderMap) {
        let bucket_id = headers
            .get("x-ratelimit-bucket")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(route)
            .to_string();

        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let reset_after = headers
            .get("x-ratelimit-reset-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let is_global = headers
            .get("x-ratelimit-global")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        let now = Instant::now();
        let resets_at = now + Duration::from_secs_f64(reset_after);

        if is_global {
            self.global_reset = Some(resets_at);
        }

        // Map route to bucket
        self.route_buckets
            .insert(route.to_string(), bucket_id.clone());

        // Update bucket info
        self.buckets.insert(
            bucket_id.clone(),
            RateLimitBucket {
                bucket_id,
                limit,
                remaining,
                resets_at,
                updated_at: now,
            },
        );
    }

    /// Check if we should wait before making a request to this route
    pub fn check_route(&self, route: &str) -> Option<Duration> {
        // Check global rate limit first
        if let Some(global_reset) = self.global_reset {
            let now = Instant::now();
            if now < global_reset {
                return Some(global_reset - now);
            }
        }

        // Check route-specific bucket
        if let Some(bucket_id) = self.route_buckets.get(route) {
            if let Some(bucket) = self.buckets.get(bucket_id) {
                return bucket.wait_time();
            }
        }

        None // No known limits — proceed
    }

    /// Pre-flight check: wait if necessary before making a request
    pub async fn wait_if_needed(&self, route: &str) {
        if let Some(wait) = self.check_route(route) {
            tracing::debug!("Rate limit: waiting {:?} before requesting {}", wait, route);
            tokio::time::sleep(wait).await;
        }
    }

    /// Normalize a route for bucket matching
    /// Discord groups routes by major parameters (guild_id, channel_id, webhook_id)
    pub fn normalize_route(method: &str, endpoint: &str) -> String {
        // Replace specific IDs with placeholders for bucket matching
        let parts: Vec<&str> = endpoint.split('/').collect();
        let mut normalized = Vec::new();

        let mut i = 0;
        while i < parts.len() {
            let part = parts[i];
            if part == "channels" || part == "guilds" || part == "webhooks" {
                normalized.push(part);
                if i + 1 < parts.len() {
                    normalized.push(parts[i + 1]); // Keep the major ID
                    i += 1;
                }
            } else if part.chars().all(|c| c.is_ascii_digit()) && part.len() > 10 {
                normalized.push(":id"); // Replace snowflake IDs
            } else {
                normalized.push(part);
            }
            i += 1;
        }

        format!("{} {}", method, normalized.join("/"))
    }

    /// Get the number of tracked buckets
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Clean up expired buckets (older than 10 minutes)
    pub fn cleanup(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(600);
        self.buckets.retain(|_, b| b.updated_at > cutoff);
        let valid_buckets: std::collections::HashSet<&String> = self.buckets.keys().collect();
        self.route_buckets
            .retain(|_, bucket_id| valid_buckets.contains(bucket_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_can_request() {
        let bucket = RateLimitBucket {
            bucket_id: "test".to_string(),
            limit: 5,
            remaining: 3,
            resets_at: Instant::now() + Duration::from_secs(10),
            updated_at: Instant::now(),
        };
        assert!(bucket.can_request());
        assert!(bucket.wait_time().is_none());
    }

    #[test]
    fn test_bucket_exhausted() {
        let bucket = RateLimitBucket {
            bucket_id: "test".to_string(),
            limit: 5,
            remaining: 0,
            resets_at: Instant::now() + Duration::from_secs(5),
            updated_at: Instant::now(),
        };
        assert!(!bucket.can_request());
        assert!(bucket.wait_time().is_some());
    }

    #[test]
    fn test_bucket_expired_allows_request() {
        let bucket = RateLimitBucket {
            bucket_id: "test".to_string(),
            limit: 5,
            remaining: 0,
            resets_at: Instant::now() - Duration::from_secs(1), // Already reset
            updated_at: Instant::now(),
        };
        assert!(bucket.can_request());
    }

    #[test]
    fn test_normalize_route() {
        assert_eq!(
            RateLimiter::normalize_route("GET", "/channels/123456789012345678/messages"),
            "GET /channels/123456789012345678/messages"
        );
        // Snowflake ID in non-major position gets replaced
        assert_eq!(
            RateLimiter::normalize_route("DELETE", "/channels/123/messages/99999999999999999"),
            "DELETE /channels/123/messages/:id"
        );
    }

    #[test]
    fn test_no_limit_returns_none() {
        let limiter = RateLimiter::new();
        assert!(limiter.check_route("/unknown/route").is_none());
    }

    #[test]
    fn test_cleanup() {
        let mut limiter = RateLimiter::new();
        limiter.buckets.insert(
            "old".to_string(),
            RateLimitBucket {
                bucket_id: "old".to_string(),
                limit: 5,
                remaining: 5,
                resets_at: Instant::now(),
                updated_at: Instant::now() - Duration::from_secs(700), // >10min old
            },
        );
        limiter.buckets.insert(
            "new".to_string(),
            RateLimitBucket {
                bucket_id: "new".to_string(),
                limit: 5,
                remaining: 5,
                resets_at: Instant::now(),
                updated_at: Instant::now(),
            },
        );
        assert_eq!(limiter.bucket_count(), 2);
        limiter.cleanup();
        assert_eq!(limiter.bucket_count(), 1);
    }
}
