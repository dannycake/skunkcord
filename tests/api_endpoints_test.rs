// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Integration tests for additional API endpoints
//!
//! Tests threads, invites, emoji, reactions, and channel management
//! against the mock Discord server.

use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn json_response(body: &serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("content-type", "application/json")
}

async fn test_client(server: &MockServer) -> discord_qt::client::DiscordClient {
    let mut client = discord_qt::client::DiscordClient::new().await.unwrap();
    client.set_api_base(format!("{}/api", server.uri()));
    client.set_token("test_token".to_string()).await;
    client
}

// ==================== Thread Tests ====================

#[tokio::test]
async fn test_list_active_threads() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/threads/active"))
        .respond_with(json_response(&serde_json::json!({
            "threads": [
                {"id": "t1", "type": 11, "name": "Thread 1", "guild_id": "g1"},
                {"id": "t2", "type": 11, "name": "Thread 2", "guild_id": "g1"}
            ],
            "members": [],
            "has_more": false
        })))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let result = client.list_active_threads("g1").await.unwrap();
    assert_eq!(result.threads.len(), 2);
    assert_eq!(result.threads[0].name.as_deref(), Some("Thread 1"));
}

#[tokio::test]
async fn test_join_thread() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v10/channels/t1/thread-members/@me"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.join_thread("t1").await.is_ok());
}

// ==================== Emoji Tests ====================

#[tokio::test]
async fn test_list_guild_emojis() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/emojis"))
        .respond_with(json_response(&serde_json::json!([
            {"id": "e1", "name": "pepe", "animated": false, "available": true},
            {"id": "e2", "name": "catjam", "animated": true, "available": true}
        ])))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let emojis = client.list_guild_emojis("g1").await.unwrap();
    assert_eq!(emojis.len(), 2);
    assert_eq!(emojis[0].name.as_deref(), Some("pepe"));
    assert_eq!(emojis[1].animated, Some(true));
}

// ==================== Channel Management Tests ====================

#[tokio::test]
async fn test_delete_channel() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/channels/ch1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.delete_channel("ch1").await.is_ok());
}

#[tokio::test]
async fn test_edit_message() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/v10/channels/ch1/messages/m1"))
        .respond_with(json_response(&serde_json::json!({
            "id": "m1",
            "channel_id": "ch1",
            "content": "edited content",
            "timestamp": "2024-01-01T00:00:00.000000+00:00",
            "edited_timestamp": "2024-01-01T01:00:00.000000+00:00",
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "attachments": [],
            "embeds": [],
            "type": 0
        })))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let edit = discord_qt::client::EditMessage {
        content: Some("edited content".to_string()),
        embeds: None,
        flags: None,
        allowed_mentions: None,
    };
    let msg = client.edit_message("ch1", "m1", &edit).await.unwrap();
    assert_eq!(msg.content, "edited content");
}

// ==================== User Operations Tests ====================

#[tokio::test]
async fn test_get_note() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me/notes/u1"))
        .respond_with(json_response(&serde_json::json!({
            "note": "Cool person"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let note = client.get_note("u1").await.unwrap();
    assert_eq!(note, Some("Cool person".to_string()));
}

#[tokio::test]
async fn test_set_note() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v10/users/@me/notes/u1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.set_note("u1", "My note").await.is_ok());
}

#[tokio::test]
async fn test_send_friend_request() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v10/users/@me/relationships"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.send_friend_request("testuser").await.is_ok());
}

#[tokio::test]
async fn test_block_user() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v10/users/@me/relationships/u1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.block_user("u1").await.is_ok());
}

// ==================== Guild Management Tests ====================

#[tokio::test]
async fn test_leave_guild() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/users/@me/guilds/g1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.leave_guild("g1").await.is_ok());
}

#[tokio::test]
async fn test_get_audit_log() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex("/api/v10/guilds/g1/audit-logs.*"))
        .respond_with(json_response(&serde_json::json!({
            "audit_log_entries": [
                {"id": "a1", "user_id": "u1", "action_type": 20, "reason": "test"}
            ],
            "users": [
                {"id": "u1", "username": "Mod", "discriminator": "0"}
            ]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let log = client.get_audit_log("g1", Some(10)).await.unwrap();
    assert_eq!(log.audit_log_entries.len(), 1);
    assert_eq!(log.users.len(), 1);
}

// ==================== Bulk Delete Test ====================

#[tokio::test]
async fn test_bulk_delete_messages() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v10/channels/ch1/messages/bulk-delete"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let ids = vec!["m1".to_string(), "m2".to_string(), "m3".to_string()];
    assert!(client.bulk_delete_messages("ch1", ids).await.is_ok());
}

// ==================== Read State / ACK Test ====================

#[tokio::test]
async fn test_ack_message() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v10/channels/ch1/messages/m99/ack"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.ack_message("ch1", "m99").await.is_ok());
}
