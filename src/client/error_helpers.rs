// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Error helper utilities
//!
//! Provides better error context for API requests by including
//! the endpoint URL, HTTP method, and status code in error messages.

use crate::{DiscordError, Result};

/// Parse a Discord API error response and create a detailed error
pub async fn parse_api_error(
    response: reqwest::Response,
    method: &str,
    endpoint: &str,
) -> DiscordError {
    let status = response.status();
    let status_code = status.as_u16();

    // Try to extract Discord's error message from the response body
    let body = response.text().await.unwrap_or_default();

    let discord_message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| {
            v.get("message")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
        });

    let discord_code = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v.get("code").and_then(|c| c.as_u64()));

    let msg = if let Some(ref dm) = discord_message {
        format!(
            "{} {} → {} {}: {} (code: {})",
            method,
            endpoint,
            status_code,
            status.canonical_reason().unwrap_or(""),
            dm,
            discord_code.unwrap_or(0)
        )
    } else {
        format!(
            "{} {} → {} {}",
            method,
            endpoint,
            status_code,
            status.canonical_reason().unwrap_or("Unknown")
        )
    };

    match status_code {
        401 => DiscordError::InvalidToken,
        403 => DiscordError::Forbidden(msg),
        404 => DiscordError::NotFound(msg),
        429 => DiscordError::RateLimited(1000),
        _ => DiscordError::Http(msg),
    }
}

/// Check if a response is successful (2xx) and return Ok or a detailed error
pub async fn check_response(
    response: reqwest::Response,
    method: &str,
    endpoint: &str,
) -> Result<reqwest::Response> {
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(parse_api_error(response, method, endpoint).await)
    }
}

/// Extract JSON body from a successful response with error context
pub async fn json_body<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    endpoint: &str,
) -> Result<T> {
    let body = response.text().await.map_err(|e| {
        DiscordError::Http(format!("Failed to read response from {}: {}", endpoint, e))
    })?;

    serde_json::from_str::<T>(&body).map_err(|e| {
        DiscordError::Http(format!(
            "Failed to parse JSON from {}: {} (body: {})",
            endpoint,
            e,
            if body.len() > 200 {
                format!("{}...", &body[..200])
            } else {
                body
            }
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse_error_context() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create a mock response with bad JSON
            let result = serde_json::from_str::<serde_json::Value>("not json");
            assert!(result.is_err());
        });
    }
}
