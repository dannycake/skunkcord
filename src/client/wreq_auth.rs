// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Credential login and MFA via wreq (Chrome TLS/HTTP2 fingerprint).
//! Used to avoid Discord/Cloudflare antibot rejecting valid MFA codes when using plain reqwest.

#![cfg(feature = "wreq-auth")]

use crate::client::session::{LoginCredentials, LoginResponse, MfaRequest, MfaResponse};
use crate::fingerprint::BrowserFingerprint;
use crate::{DiscordError, Result, API_VERSION};
use wreq::header::CONTENT_TYPE;
use wreq_util::Emulation;

const API_BASE: &str = "https://discord.com/api";

/// Build a wreq client with Chrome TLS/HTTP2 emulation and cookie store.
/// Same logical setup as our reqwest client (cookies, compression) but with browser-like TLS.
pub fn build_wreq_client() -> Result<wreq::Client> {
    let client = wreq::Client::builder()
        .emulation(Emulation::Chrome131)
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .build()
        .map_err(|e| DiscordError::Http(format!("wreq client: {}", e)))?;
    Ok(client)
}

/// Apply Discord fingerprint and auth headers to a wreq request.
fn apply_headers(
    request: wreq::RequestBuilder,
    fingerprint: &BrowserFingerprint,
    x_fingerprint: &str,
    referer: &str,
) -> wreq::RequestBuilder {
    let mut req = request
        .header("Origin", "https://discord.com")
        .header("Referer", referer)
        .header("X-Fingerprint", x_fingerprint)
        .header(CONTENT_TYPE, "application/json");
    for (key, value) in fingerprint.get_headers() {
        req = req.header(key, value.as_str());
    }
    req
}

/// POST /auth/login using wreq (Chrome TLS). Returns login response; cookies are stored in client.
pub async fn login_with_credentials_wreq(
    client: &wreq::Client,
    fingerprint: &BrowserFingerprint,
    email: &str,
    password: &str,
    x_fingerprint: &str,
    captcha_key: Option<&str>,
    captcha_rqtoken: Option<&str>,
    captcha_session_id: Option<&str>,
) -> Result<LoginResponse> {
    let url = format!("{}/v{}/auth/login", API_BASE, API_VERSION);
    let body = LoginCredentials {
        login: email.to_string(),
        password: password.to_string(),
        undelete: false,
        login_source: None,
        gift_code_sku_id: None,
        captcha_key: captcha_key.map(|s| s.to_string()),
        captcha_rqtoken: captcha_rqtoken.map(|s| s.to_string()),
    };
    let body_str = serde_json::to_string(&body)
        .map_err(|e| DiscordError::Http(format!("login body: {}", e)))?;

    let request = apply_headers(
        client.post(&url).body(body_str),
        fingerprint,
        x_fingerprint,
        "https://discord.com/login",
    );

    let response = request
        .send()
        .await
        .map_err(|e| DiscordError::Http(format!("login request: {}", e)))?;

    let status = response.status().as_u16();
    let resp_body = response
        .text()
        .await
        .map_err(|e| DiscordError::Http(format!("login response body: {}", e)))?;

    if status == 400 {
        tracing::warn!("Login 400 response: {}", resp_body);
        crate::client::captcha_interceptor::check_for_captcha(status, &resp_body)?;
        let msg = serde_json::from_str::<serde_json::Value>(&resp_body)
            .ok()
            .and_then(|v| {
                // Try to extract nested error message from errors.*._ errors[0].message
                if let Some(errors) = v.get("errors").and_then(|e| e.as_object()) {
                    for (_field, field_errors) in errors {
                        if let Some(err_list) = field_errors.get("_errors").and_then(|e| e.as_array()) {
                            if let Some(first) = err_list.first() {
                                if let Some(msg) = first.get("message").and_then(|m| m.as_str()) {
                                    return Some(msg.to_string());
                                }
                            }
                        }
                    }
                }
                v.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())
            })
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
        DiscordError::Http(format!(
            "Login response parse: {} (body: {}...)",
            e,
            &resp_body[..resp_body.len().min(100)]
        ))
    })
}

/// POST /auth/mfa/totp using wreq (same client so cookies from login are sent).
pub async fn verify_mfa_totp_wreq(
    client: &wreq::Client,
    fingerprint: &BrowserFingerprint,
    ticket: &str,
    code: &str,
    x_fingerprint: &str,
    login_instance_id: Option<&str>,
) -> Result<MfaResponse> {
    let url = format!("{}/v{}/auth/mfa/totp", API_BASE, API_VERSION);
    let body = MfaRequest {
        code: code.to_string(),
        ticket: ticket.to_string(),
        login_source: None,
        gift_code_sku_id: None,
        login_instance_id: login_instance_id.map(String::from),
    };
    let body_str = serde_json::to_string(&body)
        .map_err(|e| DiscordError::Http(format!("mfa body: {}", e)))?;

    let request = apply_headers(
        client.post(&url).body(body_str),
        fingerprint,
        x_fingerprint,
        "https://discord.com/login",
    );

    let response = request
        .send()
        .await
        .map_err(|e| DiscordError::Http(format!("mfa request: {}", e)))?;

    let status = response.status().as_u16();
    let resp_body = response
        .text()
        .await
        .map_err(|e| DiscordError::Http(format!("mfa response body: {}", e)))?;

    if status < 200 || status >= 300 {
        if status == 400 {
            let _ = crate::client::captcha_interceptor::check_for_captcha(status, &resp_body);
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
        DiscordError::Http(format!(
            "MFA response parse: {} (body: {}...)",
            e,
            &resp_body[..resp_body.len().min(100)]
        ))
    })
}
