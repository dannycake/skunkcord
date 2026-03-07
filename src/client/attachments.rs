// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! File upload and attachment handling
//!
//! Discord uses a two-step upload process for large files:
//! 1. Request upload URLs via POST /channels/{id}/attachments
//! 2. Upload file to the returned URL (GCS/CloudFlare)
//! 3. Reference the uploaded file in the message payload
//!
//! For simpler uploads, multipart form data can be used directly
//! with the message create endpoint.

use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Attachment metadata for upload request
#[derive(Debug, Clone, Serialize)]
pub struct AttachmentUploadRequest {
    pub files: Vec<AttachmentFile>,
}

/// A single file to upload
#[derive(Debug, Clone, Serialize)]
pub struct AttachmentFile {
    /// Unique ID for this attachment in the request (0-indexed)
    pub id: String,
    /// Original filename
    pub filename: String,
    /// File size in bytes
    pub file_size: u64,
}

/// Response from the attachment upload URL request
#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentUploadResponse {
    pub attachments: Vec<AttachmentUploadUrl>,
}

/// Upload URL for a single attachment
#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentUploadUrl {
    pub id: u64,
    pub upload_url: String,
    pub upload_filename: String,
}

/// Attachment reference for including in a message
#[derive(Debug, Clone, Serialize)]
pub struct AttachmentReference {
    pub id: String,
    pub filename: String,
    pub uploaded_filename: String,
}

/// Message with attachments
#[derive(Debug, Clone, Serialize)]
pub struct MessageWithAttachments {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub attachments: Vec<AttachmentReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

/// Build a multipart form body for sending a message with file attachments
pub fn build_multipart_message(
    content: Option<&str>,
    files: &[(String, Vec<u8>, String)], // (filename, bytes, content_type)
) -> reqwest::multipart::Form {
    let mut form = reqwest::multipart::Form::new();

    // Add the JSON payload part
    let mut payload = serde_json::json!({});
    if let Some(c) = content {
        payload["content"] = serde_json::json!(c);
    }

    // Add attachment references
    let attachments: Vec<serde_json::Value> = files
        .iter()
        .enumerate()
        .map(|(i, (name, _, _))| {
            serde_json::json!({
                "id": i,
                "filename": name,
            })
        })
        .collect();
    if !attachments.is_empty() {
        payload["attachments"] = serde_json::json!(attachments);
    }

    form = form.text(
        "payload_json",
        serde_json::to_string(&payload).unwrap_or_default(),
    );

    // Add file parts
    for (i, (filename, bytes, content_type)) in files.iter().enumerate() {
        let part = reqwest::multipart::Part::bytes(bytes.clone())
            .file_name(filename.clone())
            .mime_str(content_type)
            .unwrap_or_else(|_| {
                reqwest::multipart::Part::bytes(bytes.clone()).file_name(filename.clone())
            });
        form = form.part(format!("files[{}]", i), part);
    }

    form
}

/// Build multipart form for create message with full payload (content, message_reference, sticker_ids, flags, nonce) and files.
/// The payload should be a JSON object; this function adds the "attachments" array for the file parts.
pub fn build_multipart_message_with_payload(
    mut payload: serde_json::Value,
    files: &[(String, Vec<u8>, String)], // (filename, bytes, content_type)
) -> reqwest::multipart::Form {
    let mut form = reqwest::multipart::Form::new();

    let attachments: Vec<serde_json::Value> = files
        .iter()
        .enumerate()
        .map(|(i, (name, _, _))| serde_json::json!({ "id": i, "filename": name }))
        .collect();
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("attachments".to_string(), serde_json::json!(attachments));
    }

    form = form.text(
        "payload_json",
        serde_json::to_string(&payload).unwrap_or_default(),
    );

    for (i, (filename, bytes, content_type)) in files.iter().enumerate() {
        let part = reqwest::multipart::Part::bytes(bytes.clone())
            .file_name(filename.clone())
            .mime_str(content_type)
            .unwrap_or_else(|_| {
                reqwest::multipart::Part::bytes(bytes.clone()).file_name(filename.clone())
            });
        form = form.part(format!("files[{}]", i), part);
    }

    form
}

/// Maximum file size for free users (25 MB)
pub const MAX_FILE_SIZE_FREE: u64 = 25 * 1024 * 1024;
/// Maximum file size for Nitro Basic (50 MB)
pub const MAX_FILE_SIZE_NITRO_BASIC: u64 = 50 * 1024 * 1024;
/// Maximum file size for Nitro (500 MB)
pub const MAX_FILE_SIZE_NITRO: u64 = 500 * 1024 * 1024;

/// Validate file size against a limit
pub fn validate_file_size(size: u64, limit: u64) -> Result<()> {
    if size > limit {
        Err(DiscordError::Http(format!(
            "File too large: {} bytes (max: {} bytes)",
            size, limit
        )))
    } else {
        Ok(())
    }
}

/// Get the MIME type for a file based on its extension
pub fn mime_from_extension(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "txt" => "text/plain",
        "json" => "application/json",
        "js" => "text/javascript",
        "py" => "text/x-python",
        "rs" => "text/x-rust",
        _ => "application/octet-stream",
    }
}

/// Check if a file is an image (for inline display)
pub fn is_image(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp")
}

/// Check if a file is a video
pub fn is_video(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "mp4" | "webm" | "mov")
}

/// Check if a file is audio
pub fn is_audio(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "mp3" | "ogg" | "wav" | "flac")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_types() {
        assert_eq!(mime_from_extension("image.png"), "image/png");
        assert_eq!(mime_from_extension("doc.pdf"), "application/pdf");
        assert_eq!(mime_from_extension("code.rs"), "text/x-rust");
        assert_eq!(
            mime_from_extension("unknown.xyz"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_is_image() {
        assert!(is_image("photo.png"));
        assert!(is_image("photo.JPG"));
        assert!(is_image("anim.gif"));
        assert!(!is_image("doc.pdf"));
        assert!(!is_image("video.mp4"));
    }

    #[test]
    fn test_is_video() {
        assert!(is_video("clip.mp4"));
        assert!(is_video("clip.webm"));
        assert!(!is_video("photo.png"));
    }

    #[test]
    fn test_is_audio() {
        assert!(is_audio("song.mp3"));
        assert!(is_audio("track.flac"));
        assert!(!is_audio("photo.png"));
    }

    #[test]
    fn test_file_size_validation() {
        assert!(validate_file_size(1000, MAX_FILE_SIZE_FREE).is_ok());
        assert!(validate_file_size(MAX_FILE_SIZE_FREE + 1, MAX_FILE_SIZE_FREE).is_err());
    }

    #[test]
    fn test_build_multipart() {
        let files = vec![(
            "test.png".to_string(),
            vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
            "image/png".to_string(),
        )];
        let form = build_multipart_message(Some("Check out this image"), &files);
        // Form is opaque but we can verify it doesn't panic
        let _ = form;
    }

    #[test]
    fn test_build_multipart_no_content() {
        let files = vec![(
            "file.txt".to_string(),
            b"hello world".to_vec(),
            "text/plain".to_string(),
        )];
        let form = build_multipart_message(None, &files);
        let _ = form;
    }
}
