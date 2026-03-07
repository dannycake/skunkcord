// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Captcha interceptor for HTTP responses
//!
//! Automatically detects captcha challenges in any Discord API response
//! (not just login) and converts them to CaptchaRequired errors with
//! the full challenge data for the widget to use.

use crate::captcha::{CaptchaChallenge, CaptchaDetection};
use crate::{DiscordError, Result};

/// Check a Discord API response for captcha challenges.
///
/// Call this on any 400 response body. If a captcha is detected,
/// returns Err(CaptchaRequired) with the serialized challenge.
/// Otherwise returns Ok(()) to indicate normal processing should continue.
pub fn check_for_captcha(status: u16, body: &str) -> Result<()> {
    // Captcha only comes on 400 Bad Request
    if status != 400 {
        return Ok(());
    }

    match CaptchaChallenge::from_response_body(body) {
        CaptchaDetection::Challenge(challenge) => {
            let challenge_json =
                serde_json::to_string(&challenge).unwrap_or_else(|_| "{}".to_string());
            Err(DiscordError::CaptchaRequired(challenge_json))
        }
        CaptchaDetection::NotCaptcha => Ok(()),
    }
}

/// Extract the CaptchaChallenge from a CaptchaRequired error
pub fn extract_challenge(error: &DiscordError) -> Option<CaptchaChallenge> {
    if let DiscordError::CaptchaRequired(json) = error {
        serde_json::from_str(json).ok()
    } else {
        None
    }
}

/// Build the headers needed to retry a request after solving a captcha
pub fn captcha_retry_headers(captcha_token: &str, rqtoken: Option<&str>) -> Vec<(String, String)> {
    let mut headers = vec![("X-Captcha-Key".to_string(), captcha_token.to_string())];

    if let Some(rqt) = rqtoken {
        headers.push(("X-Captcha-Rqtoken".to_string(), rqt.to_string()));
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_captcha_on_200() {
        assert!(check_for_captcha(200, "{}").is_ok());
    }

    #[test]
    fn test_no_captcha_on_normal_400() {
        let body = r#"{"message": "Invalid Form Body", "code": 50035}"#;
        assert!(check_for_captcha(400, body).is_ok());
    }

    #[test]
    fn test_captcha_detected_on_400() {
        let body = r#"{
            "captcha_sitekey": "abc123",
            "captcha_service": "hcaptcha",
            "captcha_rqdata": "blob"
        }"#;
        let result = check_for_captcha(400, body);
        assert!(result.is_err());
        match result.unwrap_err() {
            DiscordError::CaptchaRequired(json) => {
                assert!(json.contains("abc123"));
            }
            _ => panic!("Expected CaptchaRequired"),
        }
    }

    #[test]
    fn test_extract_challenge() {
        let body = r#"{
            "captcha_sitekey": "key123",
            "captcha_service": "hcaptcha"
        }"#;
        let err = check_for_captcha(400, body).unwrap_err();
        let challenge = extract_challenge(&err).unwrap();
        assert_eq!(challenge.captcha_sitekey, "key123");
    }

    #[test]
    fn test_extract_from_non_captcha_error() {
        let err = DiscordError::Http("not a captcha".to_string());
        assert!(extract_challenge(&err).is_none());
    }

    #[test]
    fn test_retry_headers() {
        let headers = captcha_retry_headers("solved_token", Some("rqtoken123"));
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, "X-Captcha-Key");
        assert_eq!(headers[0].1, "solved_token");
        assert_eq!(headers[1].0, "X-Captcha-Rqtoken");
    }

    #[test]
    fn test_retry_headers_no_rqtoken() {
        let headers = captcha_retry_headers("solved_token", None);
        assert_eq!(headers.len(), 1);
    }
}
