// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Login window implementation
//!
//! Handles the web-based login flow and token extraction.

use qmetaobject::prelude::*;
use std::collections::HashMap;

/// Login window controller
#[derive(QObject, Default)]
pub struct LoginWindow {
    base: qt_base_class!(trait QObject),

    /// Whether the login window is visible
    visible: qt_property!(bool; NOTIFY visibility_changed),
    /// Loading state
    is_loading: qt_property!(bool; NOTIFY loading_changed),
    /// Error message
    error_message: qt_property!(QString; NOTIFY error_changed),
    /// Current URL in the web view
    current_url: qt_property!(QString; NOTIFY url_changed),

    /// Visibility changed signal
    visibility_changed: qt_signal!(),
    /// Loading state changed
    loading_changed: qt_signal!(),
    /// Error changed
    error_changed: qt_signal!(),
    /// URL changed
    url_changed: qt_signal!(),
    /// Login successful
    login_success: qt_signal!(token: QString, user_agent: QString, cookies: QString),
    /// Login cancelled
    login_cancelled: qt_signal!(),

    /// Show the login window
    show: qt_method!(fn(&mut self)),
    /// Hide the login window
    hide: qt_method!(fn(&mut self)),
    /// Handle URL change from web view
    on_url_changed: qt_method!(fn(&mut self, url: QString)),
    /// Handle navigation request
    on_navigation_request: qt_method!(fn(&self, url: QString) -> bool),
    /// Inject token extraction script
    inject_extraction_script: qt_method!(fn(&self) -> QString),
    /// Process extracted data
    process_extracted_data: qt_method!(fn(&mut self, data: QString)),
}

impl LoginWindow {
    /// Show the login window
    fn show(&mut self) {
        self.visible = true;
        self.is_loading = true;
        self.error_message = QString::default();
        self.visibility_changed();
        self.loading_changed();
    }

    /// Hide the login window
    fn hide(&mut self) {
        self.visible = false;
        self.visibility_changed();
    }

    /// Handle URL change
    fn on_url_changed(&mut self, url: QString) {
        self.current_url = url.clone();
        self.url_changed();

        let url_str = url.to_string();

        // Check if we're on the app page (logged in)
        if url_str.contains("/channels") || url_str.contains("/app") {
            self.is_loading = false;
            self.loading_changed();
        }
    }

    /// Handle navigation request
    fn on_navigation_request(&self, url: QString) -> bool {
        let url_str = url.to_string();

        // Block navigation to external sites
        if !url_str.contains("discord.com")
            && !url_str.contains("discordapp.com")
            && !url_str.starts_with("about:")
        {
            return false;
        }

        true
    }

    /// Get the token extraction script
    fn inject_extraction_script(&self) -> QString {
        QString::from(TOKEN_EXTRACTION_SCRIPT)
    }

    /// Process extracted data from web view
    fn process_extracted_data(&mut self, data: QString) {
        let data_str = data.to_string();

        if let Ok(extracted) = serde_json::from_str::<ExtractedLoginData>(&data_str) {
            if let Some(token) = extracted.token {
                let cookies_json =
                    serde_json::to_string(&extracted.cookies).unwrap_or_else(|_| "{}".to_string());

                self.login_success(
                    QString::from(token.as_str()),
                    QString::from(extracted.user_agent.as_str()),
                    QString::from(cookies_json.as_str()),
                );

                self.hide();
            }
        }
    }
}

/// JavaScript to inject for token extraction
pub const TOKEN_EXTRACTION_SCRIPT: &str = r#"
(function() {
    // Function to extract token from localStorage or webpack modules
    function extractToken() {
        let token = null;
        
        // Method 1: Try localStorage
        try {
            token = window.localStorage.getItem('token');
            if (token) {
                token = JSON.parse(token);
            }
        } catch (e) {}
        
        // Method 2: Try to find token in webpack modules
        if (!token) {
            try {
                const iframe = document.createElement('iframe');
                iframe.style.display = 'none';
                document.body.appendChild(iframe);
                
                const localStorage = iframe.contentWindow.localStorage;
                token = localStorage.getItem('token');
                if (token) {
                    token = JSON.parse(token);
                }
                
                document.body.removeChild(iframe);
            } catch (e) {}
        }
        
        // Method 3: Webpack module search
        if (!token && window.webpackChunkdiscord_app) {
            try {
                let m = [];
                window.webpackChunkdiscord_app.push([
                    ['__extract_token__'],
                    {},
                    (e) => {
                        m = Object.values(e.c);
                    }
                ]);
                
                for (let mod of m) {
                    if (mod?.exports?.default?.getToken) {
                        token = mod.exports.default.getToken();
                        break;
                    }
                    if (mod?.exports?.getToken) {
                        token = mod.exports.getToken();
                        break;
                    }
                }
            } catch (e) {}
        }
        
        return token;
    }
    
    // Function to extract cookies
    function extractCookies() {
        const cookies = {};
        document.cookie.split(';').forEach(function(cookie) {
            const parts = cookie.trim().split('=');
            if (parts.length >= 2) {
                cookies[parts[0]] = parts.slice(1).join('=');
            }
        });
        return cookies;
    }
    
    // Function to extract fingerprint data from local storage
    function extractFingerprint() {
        const fingerprint = {};
        try {
            fingerprint.fingerprint = localStorage.getItem('fingerprint');
            fingerprint.deviceId = localStorage.getItem('deviceId') || 
                                   localStorage.getItem('client_uuid');
        } catch (e) {}
        return fingerprint;
    }
    
    // Wait for page to fully load and user to be logged in
    function waitForLogin(callback, maxAttempts = 30) {
        let attempts = 0;
        
        function check() {
            attempts++;
            
            // Check if we're on the app page
            if (window.location.pathname.startsWith('/channels') || 
                window.location.pathname.startsWith('/app')) {
                
                const token = extractToken();
                if (token) {
                    callback({
                        success: true,
                        token: token,
                        cookies: extractCookies(),
                        fingerprint: extractFingerprint(),
                        user_agent: navigator.userAgent,
                        url: window.location.href
                    });
                    return;
                }
            }
            
            if (attempts < maxAttempts) {
                setTimeout(check, 1000);
            } else {
                callback({
                    success: false,
                    error: 'Timeout waiting for login'
                });
            }
        }
        
        check();
    }
    
    // Start extraction
    waitForLogin(function(result) {
        // Send result back to Qt
        if (window.qt && window.qt.webChannelTransport) {
            new QWebChannel(qt.webChannelTransport, function(channel) {
                channel.objects.loginHandler.onDataExtracted(JSON.stringify(result));
            });
        } else {
            // Fallback: Use title change to communicate
            document.title = 'DISCORD_TOKEN_DATA:' + JSON.stringify(result);
        }
    });
})();
"#;

/// Extracted login data structure
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExtractedLoginData {
    pub success: bool,
    pub token: Option<String>,
    pub cookies: HashMap<String, String>,
    pub fingerprint: Option<FingerprintData>,
    pub user_agent: String,
    pub url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FingerprintData {
    pub fingerprint: Option<String>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

/// Discord login URL
pub const DISCORD_LOGIN_URL: &str = "https://discord.com/login";

/// Discord app URL (indicates successful login)
pub const DISCORD_APP_URL: &str = "https://discord.com/channels/@me";
