// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Integration tests for advanced API endpoints
//!
//! Tests webhooks, stage, automod, scheduled events, forums
//! against the mock Discord server.

use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn json_response(body: &serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("content-type", "application/json")
}

async fn test_client(server: &MockServer) -> skunkcord::client::DiscordClient {
    let mut client = skunkcord::client::DiscordClient::new().await.unwrap();
    client.set_api_base(format!("{}/api", server.uri()));
    client.set_token("test_token".to_string()).await;
    client
}

// ==================== Webhook Tests ====================

#[tokio::test]
async fn test_get_channel_webhooks() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/channels/ch1/webhooks"))
        .respond_with(json_response(&serde_json::json!([
            {"id": "wh1", "type": 1, "channel_id": "ch1", "name": "My Webhook", "token": "tok123"}
        ])))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let hooks = client.get_channel_webhooks("ch1").await.unwrap();
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].name, Some("My Webhook".to_string()));
}

#[tokio::test]
async fn test_delete_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/webhooks/wh1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.delete_webhook("wh1").await.is_ok());
}

// ==================== Stage Tests ====================

#[tokio::test]
async fn test_get_stage_instance() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/stage-instances/ch1"))
        .respond_with(json_response(&serde_json::json!({
            "id": "si1",
            "guild_id": "g1",
            "channel_id": "ch1",
            "topic": "Live Q&A",
            "privacy_level": 2
        })))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let stage = client.get_stage_instance("ch1").await.unwrap();
    assert_eq!(stage.topic, "Live Q&A");
    assert_eq!(stage.privacy_level, 2);
}

#[tokio::test]
async fn test_delete_stage_instance() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/stage-instances/ch1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.delete_stage_instance("ch1").await.is_ok());
}

// ==================== AutoMod Tests ====================

#[tokio::test]
async fn test_list_automod_rules() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/auto-moderation/rules"))
        .respond_with(json_response(&serde_json::json!([
            {
                "id": "r1",
                "guild_id": "g1",
                "name": "Block Slurs",
                "creator_id": "u1",
                "event_type": 1,
                "trigger_type": 1,
                "trigger_metadata": {"keyword_filter": ["badword"]},
                "actions": [{"type": 1}],
                "enabled": true,
                "exempt_roles": [],
                "exempt_channels": []
            }
        ])))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let rules = client.list_automod_rules("g1").await.unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "Block Slurs");
    assert!(rules[0].enabled);
}

#[tokio::test]
async fn test_delete_automod_rule() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/guilds/g1/auto-moderation/rules/r1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.delete_automod_rule("g1", "r1").await.is_ok());
}

// ==================== Scheduled Events Tests ====================

#[tokio::test]
async fn test_list_scheduled_events() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex("/api/v10/guilds/g1/scheduled-events.*"))
        .respond_with(json_response(&serde_json::json!([
            {
                "id": "ev1",
                "guild_id": "g1",
                "name": "Movie Night",
                "scheduled_start_time": "2025-06-01T20:00:00+00:00",
                "privacy_level": 2,
                "status": 1,
                "entity_type": 2,
                "user_count": 15
            }
        ])))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let events = client.list_scheduled_events("g1").await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "Movie Night");
    assert_eq!(events[0].user_count, Some(15));
}

// ==================== Timeout Test ====================

#[tokio::test]
async fn test_timeout_member() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/v10/guilds/g1/members/u1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let result = client
        .timeout_member(
            "g1",
            "u1",
            Some("2025-12-01T00:00:00+00:00"),
            Some("being annoying"),
        )
        .await;
    assert!(result.is_ok());
}

// ==================== Sticker Tests ====================

#[tokio::test]
async fn test_get_sticker() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/stickers/s1"))
        .respond_with(json_response(&serde_json::json!({
            "id": "s1",
            "name": "Cool Sticker",
            "type": 2,
            "format_type": 1,
            "guild_id": "g1"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let sticker = client.get_sticker("s1").await.unwrap();
    assert_eq!(sticker.name, "Cool Sticker");
}

// ==================== Relationship Tests ====================

#[tokio::test]
async fn test_get_relationships() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me/relationships"))
        .respond_with(json_response(&serde_json::json!([
            {
                "id": "u2",
                "type": 1,
                "user": {"id": "u2", "username": "Friend", "discriminator": "0"}
            }
        ])))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let rels = client.get_relationships().await.unwrap();
    assert_eq!(rels.len(), 1);
    assert!(rels[0].is_friend());
}

// ==================== Pin Tests ====================

#[tokio::test]
async fn test_get_pinned_messages() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/channels/ch1/pins"))
        .respond_with(json_response(&serde_json::json!([
            {
                "id": "m1",
                "channel_id": "ch1",
                "content": "Pinned message",
                "timestamp": "2024-01-01T00:00:00+00:00",
                "tts": false,
                "mention_everyone": false,
                "mentions": [],
                "attachments": [],
                "embeds": [],
                "type": 0
            }
        ])))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    let pins = client.get_pinned_messages("ch1").await.unwrap();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].content, "Pinned message");
}

#[tokio::test]
async fn test_pin_message() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v10/channels/ch1/pins/m1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = test_client(&server).await;
    assert!(client.pin_message("ch1", "m1").await.is_ok());
}
