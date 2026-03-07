// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Integration tests for remaining API endpoints
//!
//! Covers polls, soundboard, forums, welcome screen, prune,
//! templates, discovery, premium, and autocomplete.

use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn json(body: &serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("content-type", "application/json")
}

async fn client(server: &MockServer) -> discord_qt::client::DiscordClient {
    let mut c = discord_qt::client::DiscordClient::new().await.unwrap();
    c.set_api_base(format!("{}/api", server.uri()));
    c.set_token("tok".to_string()).await;
    c
}

#[tokio::test]
async fn test_end_poll() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v10/channels/ch1/polls/m1/expire"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let c = client(&server).await;
    assert!(c.end_poll("ch1", "m1").await.is_ok());
}

#[tokio::test]
async fn test_list_guild_sounds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/soundboard-sounds"))
        .respond_with(json(&serde_json::json!({
            "items": [
                {"name": "airhorn", "sound_id": "s1", "volume": 1.0, "available": true}
            ]
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let sounds = c.list_guild_sounds("g1").await.unwrap();
    assert_eq!(sounds.len(), 1);
    assert_eq!(sounds[0].name, "airhorn");
}

#[tokio::test]
async fn test_create_forum_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v10/channels/ch1/threads"))
        .respond_with(json(&serde_json::json!({
            "id": "t1",
            "type": 11,
            "name": "My Post",
            "guild_id": "g1"
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let post = discord_qt::client::forums::CreateForumPost {
        name: "My Post".to_string(),
        auto_archive_duration: None,
        rate_limit_per_user: None,
        message: discord_qt::client::forums::ForumPostMessage {
            content: Some("Hello forum!".to_string()),
            embeds: None,
            flags: None,
        },
        applied_tags: None,
    };
    let ch = c.create_forum_post("ch1", &post).await.unwrap();
    assert_eq!(ch.name, Some("My Post".to_string()));
}

#[tokio::test]
async fn test_get_welcome_screen() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/welcome-screen"))
        .respond_with(json(&serde_json::json!({
            "description": "Welcome!",
            "welcome_channels": [
                {"channel_id": "ch1", "description": "Say hi", "emoji_name": "👋"}
            ]
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let screen = c.get_welcome_screen("g1").await.unwrap();
    assert_eq!(screen.description, Some("Welcome!".to_string()));
    assert_eq!(screen.welcome_channels.len(), 1);
}

#[tokio::test]
async fn test_get_prune_count() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex("/api/v10/guilds/g1/prune.*"))
        .respond_with(json(&serde_json::json!({"pruned": 42})))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let count = c.get_prune_count("g1", 7).await.unwrap();
    assert_eq!(count, 42);
}

#[tokio::test]
async fn test_get_template() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/templates/abc123"))
        .respond_with(json(&serde_json::json!({
            "code": "abc123",
            "name": "My Template",
            "usage_count": 5,
            "creator_id": "u1",
            "created_at": "2024-01-01T00:00:00+00:00",
            "updated_at": "2024-06-01T00:00:00+00:00",
            "source_guild_id": "g1",
            "serialized_source_guild": {}
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let tpl = c.get_template("abc123").await.unwrap();
    assert_eq!(tpl.name, "My Template");
    assert_eq!(tpl.usage_count, 5);
}

#[tokio::test]
async fn test_get_guild_preview() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/preview"))
        .respond_with(json(&serde_json::json!({
            "id": "g1",
            "name": "Big Server",
            "description": "A cool server",
            "features": ["DISCOVERABLE"],
            "approximate_member_count": 50000,
            "approximate_presence_count": 12000,
            "emojis": [],
            "stickers": []
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let preview = c.get_guild_preview("g1").await.unwrap();
    assert_eq!(preview.name, "Big Server");
    assert_eq!(preview.approximate_member_count, 50000);
}

#[tokio::test]
async fn test_get_vanity_url() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/g1/vanity-url"))
        .respond_with(json(&serde_json::json!({
            "code": "coolserver",
            "uses": 1234
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let vanity = c.get_vanity_url("g1").await.unwrap();
    assert_eq!(vanity.code, Some("coolserver".to_string()));
    assert_eq!(vanity.uses, Some(1234));
}

#[tokio::test]
async fn test_get_user_settings() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me/settings"))
        .respond_with(json(&serde_json::json!({
            "theme": "dark",
            "status": "online",
            "developer_mode": true,
            "locale": "en-US"
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let settings = c.get_full_settings().await.unwrap();
    assert_eq!(settings.theme, Some("dark".to_string()));
    assert_eq!(settings.developer_mode, Some(true));
}

#[tokio::test]
async fn test_create_guild_role() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v10/guilds/g1/roles"))
        .respond_with(json(&serde_json::json!({
            "id": "r1",
            "name": "Admin",
            "color": 16711680,
            "hoist": true,
            "position": 5,
            "permissions": "8",
            "managed": false,
            "mentionable": true
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let role = c
        .create_guild_role("g1", "Admin", Some(16711680), Some("8"))
        .await
        .unwrap();
    assert_eq!(role.name, "Admin");
    assert_eq!(role.color, 16711680);
}

#[tokio::test]
async fn test_typing_indicator() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v10/channels/ch1/typing"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let c = client(&server).await;
    assert!(c.trigger_typing("ch1").await.is_ok());
}

#[tokio::test]
async fn test_get_gateway_url() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/gateway"))
        .respond_with(json(&serde_json::json!({
            "url": "wss://gateway.discord.gg"
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let url = c.get_gateway().await.unwrap();
    assert_eq!(url, "wss://gateway.discord.gg");
}

// ==================== Convenience Helper Tests ====================

#[tokio::test]
async fn test_get_json_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me/guilds"))
        .respond_with(json(&serde_json::json!([
            {"id": "g1", "name": "Guild", "icon": null, "owner": false, "permissions": "0", "features": []}
        ])))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let guilds: Vec<discord_qt::client::Guild> = c.get_json("/users/@me/guilds").await.unwrap();
    assert_eq!(guilds.len(), 1);
    assert_eq!(guilds[0].name, "Guild");
}

#[tokio::test]
async fn test_get_json_404_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v10/users/999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "message": "Unknown User", "code": 10013
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let result: Result<discord_qt::client::User, _> = c.get_json("/users/999").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        discord_qt::DiscordError::NotFound(msg) => assert!(msg.contains("/users/999")),
        e => panic!("Expected NotFound, got {:?}", e),
    }
}

#[tokio::test]
async fn test_delete_ok_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v10/channels/ch1/messages/m1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let c = client(&server).await;
    assert!(c.delete_ok("/channels/ch1/messages/m1").await.is_ok());
}

#[tokio::test]
async fn test_delete_ok_failure() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v10/channels/ch1/messages/m1"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "Missing Permissions"
        })))
        .mount(&server)
        .await;

    let c = client(&server).await;
    let result = c.delete_ok("/channels/ch1/messages/m1").await;
    assert!(result.is_err());
}
