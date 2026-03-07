// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Gateway connection health monitoring
//!
//! Tracks heartbeat latency, connection uptime, and detects
//! zombie connections (no heartbeat ACK received).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Maximum number of latency samples to keep for averaging
const MAX_LATENCY_SAMPLES: usize = 20;

/// If no heartbeat ACK is received within this multiple of the
/// heartbeat interval, the connection is considered zombie
const ZOMBIE_THRESHOLD_MULTIPLIER: f64 = 2.5;

/// Gateway connection health tracker
#[derive(Debug, Clone)]
pub struct GatewayHealth {
    /// When the connection was established
    connected_at: Option<Instant>,
    /// Last heartbeat sent time
    last_heartbeat_sent: Option<Instant>,
    /// Last heartbeat ACK received time
    last_heartbeat_ack: Option<Instant>,
    /// Recent latency samples (heartbeat round-trip times)
    latency_samples: VecDeque<Duration>,
    /// Heartbeat interval from Discord
    heartbeat_interval: Option<Duration>,
    /// Total heartbeats sent
    heartbeats_sent: u64,
    /// Total heartbeat ACKs received
    heartbeats_acked: u64,
    /// Number of times we've reconnected
    reconnect_count: u32,
}

impl GatewayHealth {
    pub fn new() -> Self {
        Self {
            connected_at: None,
            last_heartbeat_sent: None,
            last_heartbeat_ack: None,
            latency_samples: VecDeque::with_capacity(MAX_LATENCY_SAMPLES),
            heartbeat_interval: None,
            heartbeats_sent: 0,
            heartbeats_acked: 0,
            reconnect_count: 0,
        }
    }

    /// Record that a connection was established
    pub fn on_connected(&mut self, heartbeat_interval_ms: u64) {
        self.connected_at = Some(Instant::now());
        self.heartbeat_interval = Some(Duration::from_millis(heartbeat_interval_ms));
    }

    /// Record that a heartbeat was sent
    pub fn on_heartbeat_sent(&mut self) {
        self.last_heartbeat_sent = Some(Instant::now());
        self.heartbeats_sent += 1;
    }

    /// Record that a heartbeat ACK was received
    pub fn on_heartbeat_ack(&mut self) {
        let now = Instant::now();
        self.last_heartbeat_ack = Some(now);
        self.heartbeats_acked += 1;

        // Calculate latency
        if let Some(sent) = self.last_heartbeat_sent {
            let latency = now.duration_since(sent);
            if self.latency_samples.len() >= MAX_LATENCY_SAMPLES {
                self.latency_samples.pop_front();
            }
            self.latency_samples.push_back(latency);
        }
    }

    /// Record a reconnection
    pub fn on_reconnect(&mut self) {
        self.reconnect_count += 1;
        self.connected_at = None;
        self.last_heartbeat_sent = None;
        self.last_heartbeat_ack = None;
    }

    /// Get the average heartbeat latency
    pub fn average_latency(&self) -> Option<Duration> {
        if self.latency_samples.is_empty() {
            return None;
        }
        let total: Duration = self.latency_samples.iter().sum();
        Some(total / self.latency_samples.len() as u32)
    }

    /// Get the most recent latency sample
    pub fn last_latency(&self) -> Option<Duration> {
        self.latency_samples.back().copied()
    }

    /// Get connection uptime
    pub fn uptime(&self) -> Option<Duration> {
        self.connected_at.map(|t| t.elapsed())
    }

    /// Check if the connection appears to be a zombie
    /// (heartbeat sent but no ACK received within threshold)
    pub fn is_zombie(&self) -> bool {
        if let (Some(sent), Some(interval)) = (self.last_heartbeat_sent, self.heartbeat_interval) {
            let threshold =
                Duration::from_secs_f64(interval.as_secs_f64() * ZOMBIE_THRESHOLD_MULTIPLIER);

            // If we sent a heartbeat but haven't gotten ACK within threshold
            if let Some(ack) = self.last_heartbeat_ack {
                // ACK is older than the last sent heartbeat, and threshold exceeded
                if ack < sent && sent.elapsed() > threshold {
                    return true;
                }
            } else {
                // Never received an ACK and threshold exceeded
                if sent.elapsed() > threshold {
                    return true;
                }
            }
        }
        false
    }

    /// Get a summary of the connection health
    pub fn summary(&self) -> HealthSummary {
        HealthSummary {
            connected: self.connected_at.is_some(),
            uptime_secs: self.uptime().map(|d| d.as_secs()),
            avg_latency_ms: self.average_latency().map(|d| d.as_millis() as u64),
            last_latency_ms: self.last_latency().map(|d| d.as_millis() as u64),
            heartbeats_sent: self.heartbeats_sent,
            heartbeats_acked: self.heartbeats_acked,
            reconnect_count: self.reconnect_count,
            is_zombie: self.is_zombie(),
        }
    }

    /// Reset all health data (on disconnect)
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for GatewayHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of gateway health for UI display
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub connected: bool,
    pub uptime_secs: Option<u64>,
    pub avg_latency_ms: Option<u64>,
    pub last_latency_ms: Option<u64>,
    pub heartbeats_sent: u64,
    pub heartbeats_acked: u64,
    pub reconnect_count: u32,
    pub is_zombie: bool,
}

impl std::fmt::Display for HealthSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.connected {
            return write!(f, "Disconnected");
        }
        write!(
            f,
            "Connected ({}ms avg, {}s uptime, {} reconnects)",
            self.avg_latency_ms.unwrap_or(0),
            self.uptime_secs.unwrap_or(0),
            self.reconnect_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_health() {
        let health = GatewayHealth::new();
        assert!(!health.is_zombie());
        assert!(health.average_latency().is_none());
        assert!(health.uptime().is_none());
    }

    #[test]
    fn test_latency_tracking() {
        let mut health = GatewayHealth::new();
        health.on_connected(41250);

        // Simulate a heartbeat cycle
        health.on_heartbeat_sent();
        std::thread::sleep(Duration::from_millis(5));
        health.on_heartbeat_ack();

        assert!(health.last_latency().is_some());
        assert!(health.last_latency().unwrap().as_millis() >= 4);
        assert!(health.average_latency().is_some());
        assert_eq!(health.heartbeats_sent, 1);
        assert_eq!(health.heartbeats_acked, 1);
    }

    #[test]
    fn test_uptime() {
        let mut health = GatewayHealth::new();
        assert!(health.uptime().is_none());

        health.on_connected(41250);
        std::thread::sleep(Duration::from_millis(10));
        assert!(health.uptime().unwrap().as_millis() >= 9);
    }

    #[test]
    fn test_reconnect_count() {
        let mut health = GatewayHealth::new();
        health.on_connected(41250);
        health.on_reconnect();
        health.on_reconnect();
        assert_eq!(health.reconnect_count, 2);
        assert!(health.connected_at.is_none());
    }

    #[test]
    fn test_summary_display() {
        let mut health = GatewayHealth::new();
        let summary = health.summary();
        assert!(!summary.connected);
        assert_eq!(format!("{}", summary), "Disconnected");

        health.on_connected(41250);
        health.on_heartbeat_sent();
        health.on_heartbeat_ack();
        let summary = health.summary();
        assert!(summary.connected);
        assert!(!summary.is_zombie);
    }

    #[test]
    fn test_max_latency_samples() {
        let mut health = GatewayHealth::new();
        health.on_connected(41250);

        for _ in 0..30 {
            health.on_heartbeat_sent();
            health.on_heartbeat_ack();
        }
        assert_eq!(health.latency_samples.len(), MAX_LATENCY_SAMPLES);
    }
}
