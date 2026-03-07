// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Polls API
//!
//! Polls are a newer Discord feature allowing users to create
//! multiple-choice polls within messages.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Poll object embedded in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    /// The question/title of the poll
    pub question: PollMedia,
    /// The available answers
    pub answers: Vec<PollAnswer>,
    /// When the poll expires (ISO 8601)
    pub expiry: Option<String>,
    /// Whether users can select multiple answers
    pub allow_multiselect: bool,
    /// Layout type
    pub layout_type: u8,
    /// Results (only present after voting or when poll ends)
    pub results: Option<PollResults>,
}

/// Poll question or answer media (text + optional emoji)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollMedia {
    pub text: Option<String>,
    pub emoji: Option<PollEmoji>,
}

/// Emoji for poll answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollEmoji {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// A single poll answer option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollAnswer {
    pub answer_id: u32,
    pub poll_media: PollMedia,
}

/// Poll results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResults {
    pub is_finalized: bool,
    pub answer_counts: Vec<PollAnswerCount>,
}

/// Vote count for a single answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollAnswerCount {
    pub id: u32,
    pub count: u32,
    pub me_voted: bool,
}

/// Create poll request (sent as part of a message)
#[derive(Debug, Clone, Serialize)]
pub struct CreatePoll {
    pub question: PollMedia,
    pub answers: Vec<CreatePollAnswer>,
    /// Duration in hours
    pub duration: u32,
    pub allow_multiselect: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_type: Option<u8>,
}

/// Answer for creating a poll
#[derive(Debug, Clone, Serialize)]
pub struct CreatePollAnswer {
    pub poll_media: PollMedia,
}

/// Poll layout types
pub mod layout_type {
    pub const DEFAULT: u8 = 1;
}

impl DiscordClient {
    /// Get voters for a specific poll answer
    pub async fn get_poll_voters(
        &self,
        channel_id: &str,
        message_id: &str,
        answer_id: u32,
    ) -> Result<Vec<super::User>> {
        let response = self
            .get(&format!(
                "/channels/{}/polls/{}/answers/{}",
                channel_id, message_id, answer_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let wrapper: serde_json::Value = serde_json::from_str(&body)?;
            if let Some(users) = wrapper.get("users") {
                let users: Vec<super::User> = serde_json::from_value(users.clone())?;
                return Ok(users);
            }
            Ok(vec![])
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get poll voters: {}",
                response.status()
            )))
        }
    }

    /// End a poll immediately (must be the poll creator)
    pub async fn end_poll(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let response = self
            .post(
                &format!("/channels/{}/polls/{}/expire", channel_id, message_id),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to end poll: {}",
                response.status()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_poll_serialization() {
        let poll = CreatePoll {
            question: PollMedia {
                text: Some("What's your favorite color?".to_string()),
                emoji: None,
            },
            answers: vec![
                CreatePollAnswer {
                    poll_media: PollMedia {
                        text: Some("Red".to_string()),
                        emoji: Some(PollEmoji {
                            id: None,
                            name: Some("🔴".to_string()),
                        }),
                    },
                },
                CreatePollAnswer {
                    poll_media: PollMedia {
                        text: Some("Blue".to_string()),
                        emoji: Some(PollEmoji {
                            id: None,
                            name: Some("🔵".to_string()),
                        }),
                    },
                },
            ],
            duration: 24,
            allow_multiselect: false,
            layout_type: None,
        };

        let json = serde_json::to_string(&poll).unwrap();
        assert!(json.contains("favorite color"));
        assert!(json.contains("Red"));
        assert!(json.contains("Blue"));
        assert!(json.contains("\"duration\":24"));
    }

    #[test]
    fn test_poll_results_deserialization() {
        let json = r#"{
            "question": {"text": "Test?"},
            "answers": [{"answer_id": 1, "poll_media": {"text": "Yes"}}],
            "allow_multiselect": false,
            "layout_type": 1,
            "results": {
                "is_finalized": true,
                "answer_counts": [{"id": 1, "count": 5, "me_voted": true}]
            }
        }"#;

        let poll: Poll = serde_json::from_str(json).unwrap();
        assert!(poll.results.is_some());
        let results = poll.results.unwrap();
        assert!(results.is_finalized);
        assert_eq!(results.answer_counts[0].count, 5);
        assert!(results.answer_counts[0].me_voted);
    }
}
