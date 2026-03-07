// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! hCaptcha handling for Discord
//!
//! Discord uses hCaptcha Enterprise for anti-automation. This module detects
//! captcha challenges on any API endpoint, generates an embeddable widget
//! with proper Enterprise parameters (rqdata/rqtoken), and handles the
//! solution flow.
//!
//! Key implementation details:
//! - captcha_sitekey is DYNAMIC — never hardcode it
//! - captcha_rqdata MUST be forwarded via hcaptcha.setData() or the token is rejected
//! - Solution goes in X-Captcha-Key header on retry
//! - captcha_rqtoken goes in X-Captcha-Rqtoken header
//!
//! # Example
//!
//! ```
//! use discord_qt::captcha::{CaptchaChallenge, CaptchaDetection};
//!
//! let body = r#"{"captcha_sitekey": "abc", "captcha_service": "hcaptcha"}"#;
//! match CaptchaChallenge::from_response_body(body) {
//!     CaptchaDetection::Challenge(c) => assert_eq!(c.captcha_sitekey, "abc"),
//!     CaptchaDetection::NotCaptcha => panic!("should detect"),
//! }
//! ```

pub mod widget;

use serde::{Deserialize, Serialize};

/// A captcha challenge received from Discord's API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    /// The captcha service (always "hcaptcha" for Discord)
    pub captcha_service: String,
    /// The hCaptcha site key — DYNAMIC, never hardcode
    pub captcha_sitekey: String,
    /// Enterprise blob data — MUST be forwarded to widget via setData()
    #[serde(default)]
    pub captcha_rqdata: Option<String>,
    /// Enterprise request token — forwarded in X-Captcha-Rqtoken on retry
    #[serde(default)]
    pub captcha_rqtoken: Option<String>,
    /// Session identifier for the challenge
    #[serde(default)]
    pub captcha_session_id: Option<String>,
    /// Error keys from a previous failed attempt
    #[serde(default)]
    pub captcha_key: Option<Vec<String>>,
}

/// State machine for captcha solving flow
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptchaState {
    /// No captcha in progress
    Idle,
    /// Challenge received, waiting to show widget
    ChallengeReceived,
    /// Widget loaded and displayed to user
    WidgetLoaded,
    /// User is actively solving the captcha
    Solving,
    /// Captcha solved, token obtained
    Solved(String),
    /// Retrying the original request with captcha token
    Retrying,
    /// Successfully completed
    Done,
    /// Captcha expired, need to re-request
    Expired,
    /// Error occurred
    Error(String),
    /// User cancelled
    Cancelled,
}

/// Result of parsing a potential captcha response
#[derive(Debug, Clone)]
pub enum CaptchaDetection {
    /// No captcha — normal response
    NotCaptcha,
    /// Captcha challenge detected
    Challenge(CaptchaChallenge),
}

impl CaptchaChallenge {
    /// Try to parse a captcha challenge from a Discord API error response body
    pub fn from_response_body(body: &str) -> CaptchaDetection {
        // Try to parse as JSON
        let json: serde_json::Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(_) => return CaptchaDetection::NotCaptcha,
        };

        // Check for captcha_sitekey field — this is the definitive indicator
        if let Some(sitekey) = json.get("captcha_sitekey").and_then(|v| v.as_str()) {
            let challenge = CaptchaChallenge {
                captcha_service: json
                    .get("captcha_service")
                    .and_then(|v| v.as_str())
                    .unwrap_or("hcaptcha")
                    .to_string(),
                captcha_sitekey: sitekey.to_string(),
                captcha_rqdata: json
                    .get("captcha_rqdata")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                captcha_rqtoken: json
                    .get("captcha_rqtoken")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                captcha_session_id: json
                    .get("captcha_session_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                captcha_key: json.get("captcha_key").and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                }),
            };
            CaptchaDetection::Challenge(challenge)
        } else {
            CaptchaDetection::NotCaptcha
        }
    }
}

/// Result of a widget title change (parsed from document.title)
#[derive(Debug, Clone)]
pub enum WidgetResult {
    /// Captcha solved successfully
    Solved(String),
    /// Captcha error
    Error(String),
    /// Captcha expired
    Expired,
    /// Unrelated title change (ignore)
    Unrelated,
}

/// Parse a document.title change from the captcha widget
pub fn parse_widget_title(title: &str) -> WidgetResult {
    if let Some(token) = title.strip_prefix("CAPTCHA_SOLVED:") {
        if token.is_empty() {
            WidgetResult::Error("Empty token received".to_string())
        } else {
            WidgetResult::Solved(token.to_string())
        }
    } else if let Some(err) = title.strip_prefix("CAPTCHA_ERROR:") {
        WidgetResult::Error(err.to_string())
    } else if title == "CAPTCHA_EXPIRED" {
        WidgetResult::Expired
    } else {
        WidgetResult::Unrelated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captcha_detection_with_sitekey() {
        let body = r#"{
            "captcha_key": ["incorrect-captcha-sol"],
            "captcha_sitekey": "f5561ba9-8f1e-40ca-9b5b-a0b3f775f58e",
            "captcha_service": "hcaptcha",
            "captcha_rqdata": "somebase64blobdata==",
            "captcha_rqtoken": "reqtokenvalue123"
        }"#;

        match CaptchaChallenge::from_response_body(body) {
            CaptchaDetection::Challenge(c) => {
                assert_eq!(c.captcha_sitekey, "f5561ba9-8f1e-40ca-9b5b-a0b3f775f58e");
                assert_eq!(c.captcha_service, "hcaptcha");
                assert_eq!(c.captcha_rqdata, Some("somebase64blobdata==".to_string()));
                assert_eq!(c.captcha_rqtoken, Some("reqtokenvalue123".to_string()));
                assert_eq!(
                    c.captcha_key,
                    Some(vec!["incorrect-captcha-sol".to_string()])
                );
            }
            CaptchaDetection::NotCaptcha => panic!("Should have detected captcha"),
        }
    }

    #[test]
    fn test_captcha_detection_minimal() {
        let body = r#"{
            "captcha_sitekey": "abc123",
            "captcha_service": "hcaptcha"
        }"#;

        match CaptchaChallenge::from_response_body(body) {
            CaptchaDetection::Challenge(c) => {
                assert_eq!(c.captcha_sitekey, "abc123");
                assert!(c.captcha_rqdata.is_none());
                assert!(c.captcha_rqtoken.is_none());
            }
            CaptchaDetection::NotCaptcha => panic!("Should have detected captcha"),
        }
    }

    #[test]
    fn test_captcha_detection_normal_error() {
        let body = r#"{"message": "401: Unauthorized", "code": 0}"#;
        match CaptchaChallenge::from_response_body(body) {
            CaptchaDetection::NotCaptcha => {} // correct
            CaptchaDetection::Challenge(_) => panic!("Should not detect captcha"),
        }
    }

    #[test]
    fn test_captcha_detection_invalid_json() {
        match CaptchaChallenge::from_response_body("not json at all") {
            CaptchaDetection::NotCaptcha => {} // correct
            CaptchaDetection::Challenge(_) => panic!("Should not detect captcha"),
        }
    }

    #[test]
    fn test_parse_widget_solved() {
        match parse_widget_title("CAPTCHA_SOLVED:P1_eyJ0eXAiOiJKV1Q=") {
            WidgetResult::Solved(token) => {
                assert_eq!(token, "P1_eyJ0eXAiOiJKV1Q=");
            }
            _ => panic!("Should be solved"),
        }
    }

    #[test]
    fn test_parse_widget_empty_token() {
        match parse_widget_title("CAPTCHA_SOLVED:") {
            WidgetResult::Error(_) => {} // correct — empty token is an error
            _ => panic!("Empty token should be error"),
        }
    }

    #[test]
    fn test_parse_widget_error() {
        match parse_widget_title("CAPTCHA_ERROR:rate-limited") {
            WidgetResult::Error(e) => assert_eq!(e, "rate-limited"),
            _ => panic!("Should be error"),
        }
    }

    #[test]
    fn test_parse_widget_expired() {
        match parse_widget_title("CAPTCHA_EXPIRED") {
            WidgetResult::Expired => {}
            _ => panic!("Should be expired"),
        }
    }

    #[test]
    fn test_parse_widget_unrelated() {
        match parse_widget_title("Discord - Login") {
            WidgetResult::Unrelated => {}
            _ => panic!("Should be unrelated"),
        }
    }
}
