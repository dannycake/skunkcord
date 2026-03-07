// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! GIF picker — Tenor search integration
//!
//! Discord uses Tenor for GIF search. This module provides the search
//! API client and data structures for the GIF picker UI.

use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Tenor API base URL (Discord uses Google's Tenor API)
const TENOR_API_BASE: &str = "https://tenor.googleapis.com/v2";

/// Tenor API key used by Discord's web client
const TENOR_API_KEY: &str = "AIzaSyAh-VwTmPDmLyOxjxr2Z6S4DwS2sjPza6s";

/// A GIF result from Tenor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifResult {
    /// Tenor GIF ID
    pub id: String,
    /// Title/description
    pub title: String,
    /// URL for the full-size GIF
    pub gif_url: String,
    /// URL for the preview/thumbnail (smaller)
    pub preview_url: String,
    /// Width of the GIF
    pub width: u32,
    /// Height of the GIF
    pub height: u32,
}

/// Tenor search response
#[derive(Debug, Clone, Deserialize)]
struct TenorSearchResponse {
    results: Vec<TenorResult>,
    #[allow(dead_code)]
    next: Option<String>,
}

/// A single Tenor result
#[derive(Debug, Clone, Deserialize)]
struct TenorResult {
    id: String,
    title: String,
    media_formats: TenorMediaFormats,
}

/// Media format variants
#[derive(Debug, Clone, Deserialize)]
struct TenorMediaFormats {
    gif: Option<TenorMedia>,
    tinygif: Option<TenorMedia>,
    mediumgif: Option<TenorMedia>,
    nanogif: Option<TenorMedia>,
}

/// A single media variant
#[derive(Debug, Clone, Deserialize)]
struct TenorMedia {
    url: String,
    dims: Option<Vec<u32>>,
    #[allow(dead_code)]
    size: Option<u64>,
}

impl TenorResult {
    fn to_gif_result(&self) -> GifResult {
        let gif = self
            .media_formats
            .mediumgif
            .as_ref()
            .or(self.media_formats.gif.as_ref());
        let preview = self
            .media_formats
            .nanogif
            .as_ref()
            .or(self.media_formats.tinygif.as_ref());

        let (gif_url, width, height) = if let Some(g) = gif {
            let dims = g.dims.as_deref().unwrap_or(&[0, 0]);
            (
                g.url.clone(),
                *dims.first().unwrap_or(&0),
                *dims.get(1).unwrap_or(&0),
            )
        } else {
            (String::new(), 0, 0)
        };

        let preview_url = preview
            .map(|p| p.url.clone())
            .unwrap_or_else(|| gif_url.clone());

        GifResult {
            id: self.id.clone(),
            title: self.title.clone(),
            gif_url,
            preview_url,
            width,
            height,
        }
    }
}

/// Search for GIFs via Tenor
pub async fn search_gifs(query: &str, limit: u8) -> Result<Vec<GifResult>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/search?q={}&key={}&client_key=discord&media_filter=gif,tinygif,mediumgif,nanogif&limit={}",
        TENOR_API_BASE,
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>(),
        TENOR_API_KEY,
        limit.min(50)
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| DiscordError::Http(format!("Tenor search failed: {}", e)))?;

    if response.status().is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;
        let tenor: TenorSearchResponse =
            serde_json::from_str(&body).map_err(|e| DiscordError::Http(e.to_string()))?;
        Ok(tenor.results.iter().map(|r| r.to_gif_result()).collect())
    } else {
        Err(DiscordError::Http(format!(
            "Tenor API error: {}",
            response.status()
        )))
    }
}

/// Get trending GIFs from Tenor
pub async fn trending_gifs(limit: u8) -> Result<Vec<GifResult>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/featured?key={}&client_key=discord&media_filter=gif,tinygif,mediumgif,nanogif&limit={}",
        TENOR_API_BASE,
        TENOR_API_KEY,
        limit.min(50)
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| DiscordError::Http(format!("Tenor trending failed: {}", e)))?;

    if response.status().is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;
        let tenor: TenorSearchResponse =
            serde_json::from_str(&body).map_err(|e| DiscordError::Http(e.to_string()))?;
        Ok(tenor.results.iter().map(|r| r.to_gif_result()).collect())
    } else {
        Err(DiscordError::Http(format!(
            "Tenor API error: {}",
            response.status()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenor_result_to_gif() {
        let result = TenorResult {
            id: "123".into(),
            title: "funny cat".into(),
            media_formats: TenorMediaFormats {
                gif: Some(TenorMedia {
                    url: "https://media.tenor.com/full.gif".into(),
                    dims: Some(vec![480, 360]),
                    size: Some(500000),
                }),
                tinygif: Some(TenorMedia {
                    url: "https://media.tenor.com/tiny.gif".into(),
                    dims: Some(vec![220, 165]),
                    size: Some(50000),
                }),
                mediumgif: None,
                nanogif: Some(TenorMedia {
                    url: "https://media.tenor.com/nano.gif".into(),
                    dims: Some(vec![90, 68]),
                    size: Some(10000),
                }),
            },
        };

        let gif = result.to_gif_result();
        assert_eq!(gif.id, "123");
        assert_eq!(gif.title, "funny cat");
        assert_eq!(gif.gif_url, "https://media.tenor.com/full.gif");
        assert_eq!(gif.preview_url, "https://media.tenor.com/nano.gif");
        assert_eq!(gif.width, 480);
        assert_eq!(gif.height, 360);
    }
}
