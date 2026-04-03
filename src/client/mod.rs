// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord HTTP Client Module
//!
//! Handles all HTTP communication with Discord's API
//! with browser fingerprint emulation. This is a user-account-only client.

pub mod account_switcher;
mod api;
pub mod attachments;
pub mod autocomplete;
pub mod automod;
pub mod captcha_interceptor;
pub mod cookies;
pub mod discovery;
pub mod entitlements;
pub mod error_helpers;
pub mod forums;
pub mod interactions;
pub mod invites;
pub mod misc_endpoints;
pub mod onboarding;
pub mod permissions;
pub mod polls;
pub mod premium;
pub mod rate_limiter;
pub mod reactions;
pub mod read_states;
pub mod role_connections;
pub mod scheduled_events;
mod session;
pub mod soundboard;
pub mod stage;
pub mod templates;
pub mod threads;
pub mod typing;
pub mod user_settings;
pub mod webhooks;
pub mod welcome_screen;
pub mod x_fingerprint;

#[cfg(feature = "wreq-auth")]
pub mod wreq_auth;

pub use api::*;
pub use cookies::DiscordCookies;
pub use invites::*;
pub use reactions::{Reaction, ReactionEmoji};
pub use session::*;
pub use threads::*;

use crate::fingerprint::BrowserFingerprint;
use crate::proxy::ProxyConfig;
use crate::{DiscordError, Result, API_VERSION};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONTENT_TYPE,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Base URL for Discord API
pub const API_BASE: &str = "https://discord.com/api";

/// Telemetry endpoints that are silently blocked (standard in all client mods)
const BLOCKED_ENDPOINTS: &[&str] = &[
    "/science",
    "/track",
    "/metrics",
    "/error-reporting",
    "/debug-logs",
];

/// Maximum number of rate-limit retries per request
const MAX_RATE_LIMIT_RETRIES: u32 = 3;

/// Discord HTTP client with browser emulation (user accounts only)
#[derive(Clone)]
pub struct DiscordClient {
    /// The reqwest HTTP client
    inner: Arc<RwLock<reqwest::Client>>,
    /// Browser fingerprint for emulation
    fingerprint: Arc<RwLock<BrowserFingerprint>>,
    /// Current session token
    token: Arc<RwLock<Option<String>>>,
    /// Session data
    session: Arc<RwLock<Option<Session>>>,
    /// Client settings
    settings: Arc<RwLock<ClientSettings>>,
    /// Proxy configuration
    proxy_config: Arc<RwLock<Option<ProxyConfig>>>,
    /// API base URL (overridable for testing)
    api_base: String,
    /// Per-route rate limit tracker (wired into HTTP methods for proactive limiting)
    rate_limiter: Arc<tokio::sync::Mutex<rate_limiter::RateLimiter>>,
}

/// Client behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSettings {
    /// Send typing indicators
    pub send_typing_indicator: bool,
    /// Show typing indicators from others
    pub show_typing_indicators: bool,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Message fetch limit
    pub message_fetch_limit: u8,
    /// Show embeds
    pub show_embeds: bool,
    /// Compact message mode
    pub compact_mode: bool,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            send_typing_indicator: true,
            show_typing_indicators: true,
            auto_reconnect: true,
            message_fetch_limit: 50,
            show_embeds: true,
            compact_mode: false,
        }
    }
}

/// Check if an endpoint is a blocked telemetry path
fn is_blocked_endpoint(endpoint: &str) -> bool {
    BLOCKED_ENDPOINTS
        .iter()
        .any(|blocked| endpoint.starts_with(blocked))
}

impl DiscordClient {
    /// Create a new Discord client with default fingerprint
    pub async fn new() -> Result<Self> {
        Self::with_fingerprint(BrowserFingerprint::new_chrome()).await
    }

    /// Create a new Discord client with a specific fingerprint
    pub async fn with_fingerprint(fingerprint: BrowserFingerprint) -> Result<Self> {
        let client = Self::build_http_client(&fingerprint, None)?;

        Ok(Self {
            inner: Arc::new(RwLock::new(client)),
            fingerprint: Arc::new(RwLock::new(fingerprint)),
            token: Arc::new(RwLock::new(None)),
            session: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(ClientSettings::default())),
            proxy_config: Arc::new(RwLock::new(None)),
            api_base: API_BASE.to_string(),
            rate_limiter: Arc::new(tokio::sync::Mutex::new(rate_limiter::RateLimiter::new())),
        })
    }

    /// Create a new Discord client with proxy configuration
    pub async fn with_proxy(fingerprint: BrowserFingerprint, proxy: ProxyConfig) -> Result<Self> {
        let client = Self::build_http_client(&fingerprint, Some(&proxy))?;

        Ok(Self {
            inner: Arc::new(RwLock::new(client)),
            fingerprint: Arc::new(RwLock::new(fingerprint)),
            token: Arc::new(RwLock::new(None)),
            session: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(ClientSettings::default())),
            proxy_config: Arc::new(RwLock::new(Some(proxy))),
            api_base: API_BASE.to_string(),
            rate_limiter: Arc::new(tokio::sync::Mutex::new(rate_limiter::RateLimiter::new())),
        })
    }

    /// Create a client pointing at a custom API base URL (for testing)
    #[cfg(test)]
    pub async fn with_base_url(base_url: &str) -> Result<Self> {
        let fingerprint = BrowserFingerprint::new_chrome();
        let client = Self::build_http_client(&fingerprint, None)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(client)),
            fingerprint: Arc::new(RwLock::new(fingerprint)),
            token: Arc::new(RwLock::new(None)),
            session: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(ClientSettings::default())),
            proxy_config: Arc::new(RwLock::new(None)),
            api_base: base_url.to_string(),
            rate_limiter: Arc::new(tokio::sync::Mutex::new(rate_limiter::RateLimiter::new())),
        })
    }

    /// Set the API base URL (for testing against mock servers)
    pub fn set_api_base(&mut self, base_url: String) {
        self.api_base = base_url;
    }

    /// Build the underlying reqwest HTTP client with browser-like headers
    fn build_http_client(
        fingerprint: &BrowserFingerprint,
        proxy: Option<&ProxyConfig>,
    ) -> Result<reqwest::Client> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        default_headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        default_headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );

        let mut builder = reqwest::Client::builder()
            .user_agent(&fingerprint.user_agent)
            .default_headers(default_headers)
            .cookie_store(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(30));

        if let Some(p) = proxy {
            if p.enabled {
                let reqwest_proxy = p
                    .to_reqwest_proxy()
                    .map_err(|e| DiscordError::Proxy(e.to_string()))?;
                builder = builder.proxy(reqwest_proxy);
            }
        }

        builder
            .build()
            .map_err(|e| DiscordError::Http(e.to_string()))
    }

    /// Set or update the proxy configuration.
    /// This rebuilds the HTTP client to apply the new proxy settings.
    pub async fn set_proxy(&self, proxy: Option<ProxyConfig>) -> Result<()> {
        let fingerprint = self.fingerprint.read().await;
        let client = Self::build_http_client(&fingerprint, proxy.as_ref())?;
        *self.inner.write().await = client;
        *self.proxy_config.write().await = proxy;
        Ok(())
    }

    /// Get current proxy configuration
    pub async fn get_proxy(&self) -> Option<ProxyConfig> {
        self.proxy_config.read().await.clone()
    }

    /// Check if proxy is currently enabled
    pub async fn is_proxy_enabled(&self) -> bool {
        self.proxy_config
            .read()
            .await
            .as_ref()
            .map(|p| p.enabled)
            .unwrap_or(false)
    }

    /// Disable proxy (rebuilds client without proxy)
    pub async fn disable_proxy(&self) -> Result<()> {
        if let Some(mut config) = self.proxy_config.read().await.clone() {
            config.enabled = false;
            self.set_proxy(Some(config)).await
        } else {
            Ok(())
        }
    }

    /// Enable proxy with current configuration (rebuilds client with proxy)
    pub async fn enable_proxy(&self) -> Result<()> {
        if let Some(mut config) = self.proxy_config.read().await.clone() {
            config.enabled = true;
            self.set_proxy(Some(config)).await
        } else {
            Err(DiscordError::Proxy(
                "No proxy configuration set".to_string(),
            ))
        }
    }

    /// Set the authentication token
    pub async fn set_token(&self, token: String) {
        // Strip "Bot " prefix if someone passes it — we only do user accounts
        let clean = if token.starts_with("Bot ") {
            token[4..].to_string()
        } else {
            token
        };
        *self.token.write().await = Some(clean);
    }

    /// Get the current token
    pub async fn get_token(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    /// Check if client is authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.token.read().await.is_some()
    }

    /// Set session data
    pub async fn set_session(&self, session: Session) {
        *self.session.write().await = Some(session);
    }

    /// Get current session
    pub async fn get_session(&self) -> Option<Session> {
        self.session.read().await.clone()
    }

    /// Update the fingerprint
    pub async fn set_fingerprint(&self, fingerprint: BrowserFingerprint) {
        *self.fingerprint.write().await = fingerprint;
    }

    /// Get the current fingerprint
    pub async fn get_fingerprint(&self) -> BrowserFingerprint {
        self.fingerprint.read().await.clone()
    }

    /// Get client settings
    pub async fn get_client_settings(&self) -> ClientSettings {
        self.settings.read().await.clone()
    }

    /// Update client settings
    pub async fn update_settings(&self, settings: ClientSettings) {
        *self.settings.write().await = settings;
    }

    /// Clear token and logout
    pub async fn logout(&self) {
        *self.token.write().await = None;
        *self.session.write().await = None;
    }

    // ==================== Internal HTTP helpers ====================

    /// Add fingerprint + auth + cookie headers to a request builder.
    /// This is called on every outgoing request to ensure we look like
    /// a real Discord web client.
    async fn prepare_request(
        &self,
        mut request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        // Browser fingerprint headers (User-Agent, X-Super-Properties, Sec-Ch-Ua, etc.)
        let fingerprint = self.fingerprint.read().await;
        for (key, value) in fingerprint.get_headers() {
            request = request.header(key, &value);
        }
        request = request
            .header("Origin", "https://discord.com")
            .header("Referer", "https://discord.com/channels/@me");

        // Authorization token (user token, sent raw — no "Bot " prefix)
        if let Some(ref token) = *self.token.read().await {
            request = request.header("Authorization", token.as_str());
        }

        // Cloudflare / Discord cookies from session
        // Missing these is a detection signal on some endpoints
        if let Some(ref session) = *self.session.read().await {
            let cookies = DiscordCookies::from_map(&session.cookies);
            let cookie_header = cookies.to_header_string();
            if !cookie_header.is_empty() {
                request = request.header("Cookie", cookie_header);
            }
        }

        request
    }

    /// Handle rate limit: if response is 429, sleep and return `true` to indicate retry.
    /// Returns `false` if the response is not a rate limit.
    async fn handle_rate_limit(response: &reqwest::Response) -> Option<Duration> {
        if response.status().as_u16() != 429 {
            return None;
        }

        // Try to get retry_after from headers first (more reliable)
        if let Some(header) = response.headers().get("retry-after") {
            if let Ok(secs_str) = header.to_str() {
                if let Ok(secs) = secs_str.parse::<f64>() {
                    return Some(Duration::from_secs_f64(secs + 0.5)); // add small jitter
                }
            }
        }

        // Fallback: 1 second
        Some(Duration::from_millis(1000))
    }

    // ==================== Rate limiting helpers ====================

    /// Proactive rate limit: wait if we already know this route is exhausted.
    async fn pre_request(&self, method: &str, endpoint: &str) {
        let route = rate_limiter::RateLimiter::normalize_route(method, endpoint);
        self.rate_limiter.lock().await.wait_if_needed(&route).await;
    }

    /// Update rate limit state from response headers so future requests
    /// can avoid 429s proactively.
    async fn post_response(&self, method: &str, endpoint: &str, response: &reqwest::Response) {
        let route = rate_limiter::RateLimiter::normalize_route(method, endpoint);
        self.rate_limiter
            .lock()
            .await
            .update_from_headers(&route, response.headers());
    }

    // ==================== Public HTTP methods ====================

    /// Make an authenticated GET request with automatic rate-limit retry
    pub async fn get(&self, endpoint: &str) -> Result<reqwest::Response> {
        // Block telemetry endpoints silently
        if is_blocked_endpoint(endpoint) {
            tracing::debug!("Blocked telemetry request to {}", endpoint);
            // Return a synthetic empty 204 response is not possible with reqwest,
            // so we return a specific error that callers can ignore
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            self.pre_request("GET", endpoint).await;

            let client = self.inner.read().await;
            let request = self.prepare_request(client.get(&url)).await;
            drop(client);

            let response = request
                .send()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            self.post_response("GET", endpoint, &response).await;

            if response.status().as_u16() == 429 {
                // Parse retry-after before consuming the response
                let retry_after = Self::handle_rate_limit(&response).await;
                // Read and discard body so connection can be reused
                let body = response
                    .text()
                    .await
                    .map_err(|e| DiscordError::Http(e.to_string()))?;

                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait = retry_after.unwrap_or(Duration::from_millis(1000));
                    tracing::warn!(
                        "Rate limited on GET {}, retry {}/{} after {:?}",
                        endpoint,
                        attempt + 1,
                        MAX_RATE_LIMIT_RETRIES,
                        wait
                    );
                    tokio::time::sleep(wait).await;
                    continue;
                } else {
                    // Parse for a better error message
                    if let Ok(rl) = serde_json::from_str::<RateLimitResponse>(&body) {
                        return Err(DiscordError::RateLimited((rl.retry_after * 1000.0) as u64));
                    }
                    return Err(DiscordError::RateLimited(1000));
                }
            }

            return Ok(response);
        }

        Err(DiscordError::RateLimited(0))
    }

    /// Make an authenticated POST request with JSON body and automatic rate-limit retry
    pub async fn post<T: Serialize + Sync>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        if is_blocked_endpoint(endpoint) {
            tracing::debug!("Blocked telemetry request to {}", endpoint);
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);
        let body_str = serde_json::to_string(body)?;

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            self.pre_request("POST", endpoint).await;

            let client = self.inner.read().await;
            let request = self
                .prepare_request(client.post(&url))
                .await
                .header(CONTENT_TYPE, "application/json")
                .body(body_str.clone());
            drop(client);

            let response = request
                .send()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            self.post_response("POST", endpoint, &response).await;

            if response.status().as_u16() == 429 {
                let retry_after = Self::handle_rate_limit(&response).await;
                let _ = response.text().await;
                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait = retry_after.unwrap_or(Duration::from_millis(1000));
                    tracing::warn!(
                        "Rate limited on POST {}, retry {}/{}",
                        endpoint,
                        attempt + 1,
                        MAX_RATE_LIMIT_RETRIES
                    );
                    tokio::time::sleep(wait).await;
                    continue;
                }
                return Err(DiscordError::RateLimited(1000));
            }

            return Ok(response);
        }

        Err(DiscordError::RateLimited(0))
    }

    /// Make an authenticated POST request with multipart form body (e.g. for message with file attachments)
    pub async fn post_multipart(
        &self,
        endpoint: &str,
        form: reqwest::multipart::Form,
    ) -> Result<reqwest::Response> {
        if is_blocked_endpoint(endpoint) {
            tracing::debug!("Blocked telemetry request to {}", endpoint);
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);
        self.pre_request("POST", endpoint).await;

        let client = self.inner.read().await;
        let request = self
            .prepare_request(client.post(&url))
            .await
            .multipart(form);
        drop(client);

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        self.post_response("POST", endpoint, &response).await;

        if response.status().as_u16() == 429 {
            let _ = response.text().await;
            return Err(DiscordError::RateLimited(1000));
        }

        Ok(response)
    }

    /// Make an authenticated POST request without body
    pub async fn post_empty(&self, endpoint: &str) -> Result<reqwest::Response> {
        if is_blocked_endpoint(endpoint) {
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            self.pre_request("POST", endpoint).await;

            let client = self.inner.read().await;
            let request = self.prepare_request(client.post(&url)).await;
            drop(client);

            let response = request
                .send()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            self.post_response("POST", endpoint, &response).await;

            if response.status().as_u16() == 429 {
                let retry_after = Self::handle_rate_limit(&response).await;
                let _ = response.text().await;
                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait = retry_after.unwrap_or(Duration::from_millis(1000));
                    tokio::time::sleep(wait).await;
                    continue;
                }
                return Err(DiscordError::RateLimited(1000));
            }

            return Ok(response);
        }

        Err(DiscordError::RateLimited(0))
    }

    /// Make an authenticated PUT request with JSON body
    pub async fn put<T: Serialize + Sync>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        if is_blocked_endpoint(endpoint) {
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);
        let body_str = serde_json::to_string(body)?;

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            self.pre_request("PUT", endpoint).await;

            let client = self.inner.read().await;
            let request = self
                .prepare_request(client.put(&url))
                .await
                .header(CONTENT_TYPE, "application/json")
                .body(body_str.clone());
            drop(client);

            let response = request
                .send()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            self.post_response("PUT", endpoint, &response).await;

            if response.status().as_u16() == 429 {
                let retry_after = Self::handle_rate_limit(&response).await;
                let _ = response.text().await;
                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait = retry_after.unwrap_or(Duration::from_millis(1000));
                    tokio::time::sleep(wait).await;
                    continue;
                }
                return Err(DiscordError::RateLimited(1000));
            }

            return Ok(response);
        }

        Err(DiscordError::RateLimited(0))
    }

    /// Make an authenticated PATCH request with JSON body
    pub async fn patch<T: Serialize + Sync>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        if is_blocked_endpoint(endpoint) {
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);
        let body_str = serde_json::to_string(body)?;

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            self.pre_request("PATCH", endpoint).await;

            let client = self.inner.read().await;
            let request = self
                .prepare_request(client.patch(&url))
                .await
                .header(CONTENT_TYPE, "application/json")
                .body(body_str.clone());
            drop(client);

            let response = request
                .send()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            self.post_response("PATCH", endpoint, &response).await;

            if response.status().as_u16() == 429 {
                let retry_after = Self::handle_rate_limit(&response).await;
                let _ = response.text().await;
                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait = retry_after.unwrap_or(Duration::from_millis(1000));
                    tokio::time::sleep(wait).await;
                    continue;
                }
                return Err(DiscordError::RateLimited(1000));
            }

            return Ok(response);
        }

        Err(DiscordError::RateLimited(0))
    }

    /// Make an authenticated DELETE request
    pub async fn delete(&self, endpoint: &str) -> Result<reqwest::Response> {
        if is_blocked_endpoint(endpoint) {
            return Err(DiscordError::TelemetryBlocked);
        }

        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            self.pre_request("DELETE", endpoint).await;

            let client = self.inner.read().await;
            let request = self.prepare_request(client.delete(&url)).await;
            drop(client);

            let response = request
                .send()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;

            self.post_response("DELETE", endpoint, &response).await;

            if response.status().as_u16() == 429 {
                let retry_after = Self::handle_rate_limit(&response).await;
                let _ = response.text().await;
                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait = retry_after.unwrap_or(Duration::from_millis(1000));
                    tokio::time::sleep(wait).await;
                    continue;
                }
                return Err(DiscordError::RateLimited(1000));
            }

            return Ok(response);
        }

        Err(DiscordError::RateLimited(0))
    }

    // ==================== Convenience helpers ====================

    /// GET an endpoint and parse the JSON response, with proper error context.
    /// This is the recommended way to call most API endpoints.
    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self.get(endpoint).await?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| DiscordError::Http(format!("GET {}: read error: {}", endpoint, e)))?;

        if status.is_success() {
            serde_json::from_str::<T>(&body).map_err(|e| {
                DiscordError::Http(format!(
                    "GET {}: JSON parse error: {} (body: {})",
                    endpoint,
                    e,
                    if body.len() > 200 {
                        &body[..200]
                    } else {
                        &body
                    }
                ))
            })
        } else {
            // Check for captcha challenge on 400 responses
            captcha_interceptor::check_for_captcha(status.as_u16(), &body)?;

            let discord_msg = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                });

            Err(match status.as_u16() {
                401 => DiscordError::InvalidToken,
                403 => DiscordError::Forbidden(format!(
                    "GET {}: {}",
                    endpoint,
                    discord_msg.unwrap_or_else(|| "Forbidden".to_string())
                )),
                404 => DiscordError::NotFound(format!("GET {}: Not Found", endpoint)),
                _ => DiscordError::Http(format!(
                    "GET {}: {} {}",
                    endpoint,
                    status.as_u16(),
                    discord_msg.unwrap_or_else(|| status.to_string())
                )),
            })
        }
    }

    /// POST an endpoint with JSON body and parse the JSON response.
    /// Automatically detects captcha challenges on 400 responses.
    pub async fn post_json<B: Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let response = self.post(endpoint, body).await?;
        let status = response.status();
        let resp_body = response
            .text()
            .await
            .map_err(|e| DiscordError::Http(format!("POST {}: read error: {}", endpoint, e)))?;

        if status.is_success() {
            serde_json::from_str::<T>(&resp_body).map_err(|e| {
                DiscordError::Http(format!(
                    "POST {}: JSON parse error: {} (body: {})",
                    endpoint,
                    e,
                    if resp_body.len() > 200 {
                        &resp_body[..200]
                    } else {
                        &resp_body
                    }
                ))
            })
        } else {
            // Check for captcha
            captcha_interceptor::check_for_captcha(status.as_u16(), &resp_body)?;

            let discord_msg = serde_json::from_str::<serde_json::Value>(&resp_body)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                });

            Err(match status.as_u16() {
                401 => DiscordError::InvalidToken,
                403 => DiscordError::Forbidden(format!(
                    "POST {}: {}",
                    endpoint,
                    discord_msg.unwrap_or_else(|| "Forbidden".to_string())
                )),
                404 => DiscordError::NotFound(format!("POST {}: Not Found", endpoint)),
                _ => DiscordError::Http(format!(
                    "POST {}: {} {}",
                    endpoint,
                    status.as_u16(),
                    discord_msg.unwrap_or_else(|| status.to_string())
                )),
            })
        }
    }

    /// DELETE an endpoint and check for success (200 or 204).
    pub async fn delete_ok(&self, endpoint: &str) -> Result<()> {
        let response = self.delete(endpoint).await?;
        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            let discord_msg = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                });
            Err(DiscordError::Http(format!(
                "DELETE {}: {} {}",
                endpoint,
                body.len(),
                discord_msg.unwrap_or_else(|| "failed".to_string())
            )))
        }
    }

    /// PUT an endpoint with body and check for success (200 or 204).
    pub async fn put_ok<B: Serialize + Sync>(&self, endpoint: &str, body: &B) -> Result<()> {
        let response = self.put(endpoint, body).await?;
        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let resp_body = response.text().await.unwrap_or_default();
            Err(DiscordError::Http(format!(
                "PUT {}: failed: {}",
                endpoint,
                if resp_body.len() > 200 {
                    &resp_body[..200]
                } else {
                    &resp_body
                }
            )))
        }
    }

    // ==================== Auth login (unauthenticated) ====================

    /// POST /auth/login with email/password. Requires X-Fingerprint from /experiments.
    /// Optionally pass captcha solution headers when retrying after captcha.
    pub async fn login_with_credentials(
        &self,
        email: &str,
        password: &str,
        x_fingerprint: &str,
        captcha_key: Option<&str>,
        captcha_rqtoken: Option<&str>,
        captcha_session_id: Option<&str>,
    ) -> Result<LoginResponse> {
        let endpoint = "/auth/login";
        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);
        let body = LoginCredentials {
            login: email.to_string(),
            password: password.to_string(),
            undelete: false,
            login_source: None,
            gift_code_sku_id: None,
        };
        let body_str = serde_json::to_string(&body)
            .map_err(|e| DiscordError::Http(format!("login body: {}", e)))?;

        self.pre_request("POST", endpoint).await;

        let client = self.inner.read().await;
        let mut request = self
            .prepare_request(client.post(&url))
            .await
            .header(CONTENT_TYPE, "application/json")
            .header("X-Fingerprint", x_fingerprint)
            .header("Referer", "https://discord.com/login")
            .body(body_str);
        if let Some(key) = captcha_key {
            request = request.header("X-Captcha-Key", key);
        }
        if let Some(rqt) = captcha_rqtoken {
            request = request.header("X-Captcha-Rqtoken", rqt);
        }
        if let Some(sid) = captcha_session_id {
            request = request.header("X-Captcha-Session-Id", sid);
        }
        drop(client);

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        self.post_response("POST", endpoint, &response).await;

        let status = response.status().as_u16();
        let resp_body = response
            .text()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if status == 400 {
            self::captcha_interceptor::check_for_captcha(status, &resp_body)?;
            let msg = serde_json::from_str::<serde_json::Value>(&resp_body)
                .ok()
                .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| "Bad Request".to_string());
            return Err(DiscordError::Http(format!("Login failed: {}", msg)));
        }

        if status < 200 || status >= 300 {
            if status == 401 {
                return Err(DiscordError::InvalidToken);
            }
            if status == 403 {
                return Err(DiscordError::Forbidden(
                    serde_json::from_str::<serde_json::Value>(&resp_body)
                        .ok()
                        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
                        .unwrap_or_else(|| "Forbidden".to_string()),
                ));
            }
            return Err(DiscordError::Http(format!(
                "Login failed: {} {}",
                status,
                if resp_body.len() > 200 {
                    &resp_body[..200]
                } else {
                    &resp_body
                }
            )));
        }

        serde_json::from_str(&resp_body).map_err(|e| {
            DiscordError::Http(format!("Login response parse: {} (body: {}...)", e, &resp_body[..resp_body.len().min(100)]))
        })
    }

    /// POST /auth/mfa/totp to complete MFA and get token.
    pub async fn verify_mfa_totp(
        &self,
        ticket: &str,
        code: &str,
        x_fingerprint: &str,
        login_instance_id: Option<&str>,
    ) -> Result<MfaResponse> {
        let endpoint = "/auth/mfa/totp";
        let url = format!("{}/v{}{}", self.api_base, API_VERSION, endpoint);
        let body = MfaRequest {
            code: code.to_string(),
            ticket: ticket.to_string(),
            login_source: None,
            gift_code_sku_id: None,
            login_instance_id: login_instance_id.map(String::from),
        };
        let body_str = serde_json::to_string(&body)
            .map_err(|e| DiscordError::Http(format!("mfa body: {}", e)))?;

        self.pre_request("POST", endpoint).await;

        let client = self.inner.read().await;
        let request = self
            .prepare_request(client.post(&url))
            .await
            .header(CONTENT_TYPE, "application/json")
            .header("X-Fingerprint", x_fingerprint)
            .header("Referer", "https://discord.com/login")
            .body(body_str);
        drop(client);

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        self.post_response("POST", endpoint, &response).await;

        let status = response.status().as_u16();
        let resp_body = response
            .text()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if status < 200 || status >= 300 {
            if status == 400 {
                let _ = self::captcha_interceptor::check_for_captcha(status, &resp_body);
            }
            return Err(DiscordError::Http(format!(
                "MFA failed: {} {}",
                status,
                if resp_body.len() > 200 {
                    &resp_body[..200]
                } else {
                    &resp_body
                }
            )));
        }

        serde_json::from_str(&resp_body).map_err(|e| {
            DiscordError::Http(format!("MFA response parse: {}", e))
        })
    }

    // ==================== Token validation ====================

    /// Validate the current token
    pub async fn validate_token(&self) -> Result<User> {
        let response = self.get("/users/@me").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let user: User = serde_json::from_str(&body)?;
            Ok(user)
        } else if response.status().as_u16() == 401 {
            Err(DiscordError::InvalidToken)
        } else {
            Err(DiscordError::Auth(format!(
                "Token validation failed: {}",
                response.status()
            )))
        }
    }
}

/// Rate limit response from Discord
#[derive(Debug, Deserialize)]
pub struct RateLimitResponse {
    pub message: String,
    pub retry_after: f64,
    pub global: bool,
}

/// Discord user object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    /// Deprecated by Discord, often "0" or omitted
    #[serde(default)]
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub banner: Option<String>,
    pub accent_color: Option<u32>,
    pub locale: Option<String>,
    pub verified: Option<bool>,
    pub email: Option<String>,
    pub flags: Option<u64>,
    pub premium_type: Option<u8>,
    pub public_flags: Option<u64>,
    pub avatar_decoration_data: Option<serde_json::Value>,
    pub bio: Option<String>,
}

impl User {
    /// Get the user's display name (global_name or username)
    pub fn display_name(&self) -> &str {
        self.global_name.as_deref().unwrap_or(&self.username)
    }

    /// Get the user's avatar URL
    pub fn avatar_url(&self, size: u32) -> String {
        if let Some(ref hash) = self.avatar {
            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}?size={}",
                self.id, hash, ext, size
            )
        } else {
            // Default avatar based on discriminator or user ID
            let index = if self.discriminator == "0" {
                (self.id.parse::<u64>().unwrap_or(0) >> 22) % 6
            } else {
                self.discriminator.parse::<u64>().unwrap_or(0) % 5
            };
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
        }
    }
}
