// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Human-like request timing jitter
//!
//! Adds random delays between sequential API requests to mimic natural
//! user behavior. Without this, automated clients send requests at
//! machine-speed intervals which is a detection signal.

use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Request timing controller that adds human-like jitter
#[derive(Clone)]
pub struct RequestTimer {
    last_request: Arc<Mutex<Option<Instant>>>,
    enabled: Arc<Mutex<bool>>,
}

impl RequestTimer {
    /// Create a new request timer
    pub fn new(enabled: bool) -> Self {
        Self {
            last_request: Arc::new(Mutex::new(None)),
            enabled: Arc::new(Mutex::new(enabled)),
        }
    }

    /// Enable or disable timing jitter
    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().await = enabled;
    }

    /// Wait an appropriate amount of time before making a request.
    /// This should be called before every API request.
    ///
    /// The delay is random between 50-300ms, but only if the previous
    /// request was very recent (< 500ms ago). This prevents obvious
    /// machine-speed request patterns while not slowing down naturally-
    /// spaced user actions.
    pub async fn wait_before_request(&self) {
        if !*self.enabled.lock().await {
            return;
        }

        // Compute the sleep duration while holding the lock, then release
        // the lock (and rng) before the await to keep the future Send-safe.
        let sleep_duration = {
            let mut last = self.last_request.lock().await;
            let dur = if let Some(prev) = *last {
                let elapsed = prev.elapsed();
                if elapsed < Duration::from_millis(500) {
                    let mut rng = rand::thread_rng();
                    let jitter_ms = rng.gen_range(50..=300);
                    let target_delay = Duration::from_millis(jitter_ms);
                    if elapsed < target_delay {
                        Some(target_delay - elapsed)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            *last = Some(Instant::now());
            dur
        };

        if let Some(dur) = sleep_duration {
            tokio::time::sleep(dur).await;
        }
    }
}

impl Default for RequestTimer {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timer_disabled_no_delay() {
        let timer = RequestTimer::new(false);
        let start = Instant::now();
        timer.wait_before_request().await;
        timer.wait_before_request().await;
        // Should be nearly instant
        assert!(start.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_timer_enabled_adds_delay_on_rapid_requests() {
        let timer = RequestTimer::new(true);
        timer.wait_before_request().await;
        let start = Instant::now();
        timer.wait_before_request().await;
        // Should have added at least 50ms delay
        assert!(start.elapsed() >= Duration::from_millis(45));
    }
}
