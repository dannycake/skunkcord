// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Web view component for Discord login
//!
//! This module provides the web view functionality for browser-based login
//! and token extraction.

use crate::fingerprint::BrowserFingerprint;
use crate::ui::login_window::{ExtractedLoginData, DISCORD_LOGIN_URL, TOKEN_EXTRACTION_SCRIPT};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Web view controller for managing the login browser
pub struct WebViewController {
    /// Channel for sending extracted data
    data_tx: mpsc::Sender<ExtractedLoginData>,
    /// Current fingerprint (retained for session reconstruction)
    #[allow(dead_code)]
    fingerprint: BrowserFingerprint,
    /// Custom user agent
    user_agent: String,
}

impl WebViewController {
    /// Create a new web view controller
    pub fn new(fingerprint: BrowserFingerprint) -> (Self, mpsc::Receiver<ExtractedLoginData>) {
        let (data_tx, data_rx) = mpsc::channel(1);
        let user_agent = fingerprint.user_agent.clone();

        (
            Self {
                data_tx,
                fingerprint,
                user_agent,
            },
            data_rx,
        )
    }

    /// Get the login URL
    pub fn login_url(&self) -> &'static str {
        DISCORD_LOGIN_URL
    }

    /// Get the user agent to use
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Get JavaScript to inject after page load
    pub fn get_injection_script(&self) -> &'static str {
        TOKEN_EXTRACTION_SCRIPT
    }

    /// Process data from the web view
    pub async fn process_data(&self, data: ExtractedLoginData) {
        let _ = self.data_tx.send(data).await;
    }

    /// Check if URL indicates successful login
    pub fn is_logged_in_url(&self, url: &str) -> bool {
        url.contains("/channels") || url.contains("/app")
    }

    /// Check if URL should be allowed
    pub fn should_allow_navigation(&self, url: &str) -> bool {
        url.contains("discord.com")
            || url.contains("discordapp.com")
            || url.starts_with("about:")
            || url.starts_with("data:")
    }
}

/// Request interceptor for modifying web requests
pub struct RequestInterceptor {
    fingerprint: BrowserFingerprint,
    /// Captured tokens from requests
    captured_tokens: Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl RequestInterceptor {
    pub fn new(fingerprint: BrowserFingerprint) -> Self {
        Self {
            fingerprint,
            captured_tokens: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Get headers to add/override for requests
    pub fn get_request_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        for (key, value) in self.fingerprint.get_headers() {
            headers.insert(key.to_string(), value);
        }

        headers
    }

    /// Check if a response contains a token and capture it
    pub async fn check_response_for_token(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        body: &str,
    ) {
        // Check Authorization header
        if let Some(auth) = headers.get("Authorization") {
            if auth.split('.').count() == 3 {
                self.captured_tokens.write().await.push(auth.clone());
            }
        }

        // Check for token in response body
        if url.contains("/api/") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                    self.captured_tokens.write().await.push(token.to_string());
                }
            }
        }
    }

    /// Get captured tokens
    pub async fn get_captured_tokens(&self) -> Vec<String> {
        self.captured_tokens.read().await.clone()
    }
}

/// Cookie manager for the web view
pub struct CookieManager {
    cookies: HashMap<String, Cookie>,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<i64>,
}

impl CookieManager {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Add or update a cookie
    pub fn set_cookie(&mut self, cookie: Cookie) {
        let key = format!("{}:{}:{}", cookie.domain, cookie.path, cookie.name);
        self.cookies.insert(key, cookie);
    }

    /// Get cookies for a URL
    pub fn get_cookies_for_url(&self, url: &str) -> Vec<&Cookie> {
        let parsed = url::Url::parse(url).ok();

        self.cookies
            .values()
            .filter(|cookie| {
                if let Some(ref parsed) = parsed {
                    let domain_match = parsed
                        .host_str()
                        .map(|h| h.ends_with(&cookie.domain) || h == &cookie.domain[1..])
                        .unwrap_or(false);

                    let path_match = parsed.path().starts_with(&cookie.path);

                    domain_match && path_match
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get all cookies as a map
    pub fn get_all_cookies(&self) -> HashMap<String, String> {
        self.cookies
            .values()
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect()
    }

    /// Clear all cookies
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Get specific Discord cookies
    pub fn get_discord_cookies(&self) -> DiscordCookies {
        let cookies = self.get_all_cookies();

        DiscordCookies {
            dcfduid: cookies.get("__dcfduid").cloned(),
            sdcfduid: cookies.get("__sdcfduid").cloned(),
            cfruid: cookies.get("__cfruid").cloned(),
            locale: cookies.get("locale").cloned(),
        }
    }
}

impl Default for CookieManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Important Discord cookies
#[derive(Debug, Clone)]
pub struct DiscordCookies {
    pub dcfduid: Option<String>,
    pub sdcfduid: Option<String>,
    pub cfruid: Option<String>,
    pub locale: Option<String>,
}

impl DiscordCookies {
    /// Format cookies as a cookie header string
    pub fn to_header_string(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref v) = self.dcfduid {
            parts.push(format!("__dcfduid={}", v));
        }
        if let Some(ref v) = self.sdcfduid {
            parts.push(format!("__sdcfduid={}", v));
        }
        if let Some(ref v) = self.cfruid {
            parts.push(format!("__cfruid={}", v));
        }
        if let Some(ref v) = self.locale {
            parts.push(format!("locale={}", v));
        }

        parts.join("; ")
    }
}

/// Network request data for token interception
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Network response data
#[derive(Debug, Clone)]
pub struct NetworkResponse {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// JavaScript for intercepting network requests
pub const NETWORK_INTERCEPT_SCRIPT: &str = r#"
(function() {
    // Store original fetch and XMLHttpRequest
    const originalFetch = window.fetch;
    const originalXHR = window.XMLHttpRequest;
    
    // Override fetch to intercept requests
    window.fetch = async function(...args) {
        const request = args[0];
        const init = args[1] || {};
        
        // Call original fetch
        const response = await originalFetch.apply(this, args);
        
        // Check for authorization header in successful API calls
        if (response.ok && (typeof request === 'string' && request.includes('/api/'))) {
            const authHeader = init.headers?.Authorization || init.headers?.authorization;
            if (authHeader) {
                // Store the token
                try {
                    localStorage.setItem('_intercepted_token', authHeader);
                } catch (e) {}
            }
        }
        
        return response;
    };
    
    // Override XMLHttpRequest
    const originalOpen = originalXHR.prototype.open;
    const originalSetRequestHeader = originalXHR.prototype.setRequestHeader;
    
    originalXHR.prototype.open = function(method, url, ...rest) {
        this._requestUrl = url;
        this._requestHeaders = {};
        return originalOpen.apply(this, [method, url, ...rest]);
    };
    
    originalXHR.prototype.setRequestHeader = function(name, value) {
        this._requestHeaders[name] = value;
        
        // Capture authorization header
        if (name.toLowerCase() === 'authorization') {
            try {
                localStorage.setItem('_intercepted_token', value);
            } catch (e) {}
        }
        
        return originalSetRequestHeader.apply(this, [name, value]);
    };
})();
"#;

/// Combined extraction script that includes network interception
pub fn get_full_extraction_script() -> String {
    format!(
        "{}\n\n{}",
        NETWORK_INTERCEPT_SCRIPT, TOKEN_EXTRACTION_SCRIPT
    )
}
