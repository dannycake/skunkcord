// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Mock Discord server for integration testing
//!
//! Uses wiremock to simulate Discord's HTTP API. This allows testing
//! the full client flow (auth, API calls, rate limits, captchas)
//! without touching Discord's real servers.

use wiremock::matchers::{header, header_exists, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper: create standard Discord API JSON response headers
fn discord_json_response(body: &serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("content-type", "application/json")
}

/// Helper: create a mock user response
fn mock_user_json(id: &str, username: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "username": username,
        "discriminator": "0",
        "global_name": username,
        "avatar": null,
        "bot": false,
        "system": false,
        "mfa_enabled": false,
        "locale": "en-US",
        "verified": true,
        "email": "test@example.com",
        "flags": 0,
        "premium_type": 0,
        "public_flags": 0
    })
}

/// Helper: create a mock guild response
fn mock_guild_json(id: &str, name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "icon": null,
        "owner": false,
        "permissions": "2199023255551",
        "features": []
    })
}

/// Helper: create a mock channel response
fn mock_channel_json(id: &str, name: &str, channel_type: u8) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "type": channel_type,
        "name": name,
        "position": 0,
        "guild_id": "guild_001"
    })
}

/// Helper: create a mock message response
fn mock_message_json(id: &str, content: &str, author_id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "channel_id": "channel_001",
        "author": mock_user_json(author_id, "TestUser"),
        "content": content,
        "timestamp": "2024-01-01T00:00:00.000000+00:00",
        "edited_timestamp": null,
        "tts": false,
        "mention_everyone": false,
        "mentions": [],
        "attachments": [],
        "embeds": [],
        "type": 0
    })
}

// ==================== Token Validation Tests ====================

#[tokio::test]
async fn test_validate_token_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me"))
        .and(header("Authorization", "test_token_123"))
        .respond_with(discord_json_response(&mock_user_json("123", "TestUser")))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("test_token_123".to_string()).await;

    let user = client.validate_token().await.unwrap();
    assert_eq!(user.id, "123");
    assert_eq!(user.username, "TestUser");
}

#[tokio::test]
async fn test_validate_token_invalid() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "message": "401: Unauthorized",
            "code": 0
        })))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("bad_token".to_string()).await;

    let result = client.validate_token().await;
    assert!(result.is_err());
}

// ==================== Guild Tests ====================

#[tokio::test]
async fn test_get_guilds() {
    let server = MockServer::start().await;

    let guilds = serde_json::json!([
        mock_guild_json("g1", "Test Guild 1"),
        mock_guild_json("g2", "Test Guild 2"),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me/guilds"))
        .respond_with(discord_json_response(&guilds))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let guilds = client.get_guilds().await.unwrap();
    assert_eq!(guilds.len(), 2);
    assert_eq!(guilds[0].name, "Test Guild 1");
    assert_eq!(guilds[1].name, "Test Guild 2");
}

// ==================== Channel Tests ====================

#[tokio::test]
async fn test_get_guild_channels() {
    let server = MockServer::start().await;

    let channels = serde_json::json!([
        mock_channel_json("ch1", "general", 0),
        mock_channel_json("ch2", "voice", 2),
    ]);

    Mock::given(method("GET"))
        .and(path("/api/v10/guilds/guild_001/channels"))
        .respond_with(discord_json_response(&channels))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let channels = client.get_guild_channels("guild_001").await.unwrap();
    assert_eq!(channels.len(), 2);
    assert!(channels[0].is_text());
    assert!(channels[1].is_voice());
}

// ==================== Message Tests ====================

#[tokio::test]
async fn test_get_messages() {
    let server = MockServer::start().await;

    let messages = serde_json::json!([
        mock_message_json("m1", "Hello!", "u1"),
        mock_message_json("m2", "World!", "u2"),
    ]);

    Mock::given(method("GET"))
        .and(path_regex("/api/v10/channels/.*/messages.*"))
        .respond_with(discord_json_response(&messages))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let messages = client.get_messages("ch1", 50).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "Hello!");
}

#[tokio::test]
async fn test_send_message() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v10/channels/ch1/messages"))
        .respond_with(discord_json_response(&mock_message_json(
            "m_new",
            "Test message",
            "me",
        )))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let msg = discord_qt::client::CreateMessage::text("Test message");
    let result = client.send_message("ch1", msg).await.unwrap();
    assert_eq!(result.content, "Test message");
    assert_eq!(result.id, "m_new");
}

// ==================== Rate Limit Tests ====================

#[tokio::test]
async fn test_rate_limit_retry() {
    let server = MockServer::start().await;

    // First request: rate limited
    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({
                    "message": "You are being rate limited.",
                    "retry_after": 0.1,
                    "global": false
                }))
                .insert_header("retry-after", "0.1"),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second request: success
    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me"))
        .respond_with(discord_json_response(&mock_user_json("123", "TestUser")))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let user = client.validate_token().await.unwrap();
    assert_eq!(user.id, "123");
}

// ==================== Telemetry Blocking Tests ====================

#[tokio::test]
async fn test_telemetry_blocked() {
    let server = MockServer::start().await;

    // This mock should NEVER be hit
    Mock::given(method("POST"))
        .and(path("/api/v10/science"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let result = client
        .post("/science", &serde_json::json!({"events": []}))
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        discord_qt::DiscordError::TelemetryBlocked => {} // Expected
        e => panic!("Expected TelemetryBlocked, got: {:?}", e),
    }
}

// ==================== Captcha Detection Tests ====================

#[tokio::test]
async fn test_captcha_detection_on_login() {
    use discord_qt::captcha::{CaptchaChallenge, CaptchaDetection};

    let captcha_response = serde_json::json!({
        "captcha_key": ["incorrect-captcha-sol"],
        "captcha_sitekey": "f5561ba9-8f1e-40ca-9b5b-a0b3f775f58e",
        "captcha_service": "hcaptcha",
        "captcha_rqdata": "dGVzdGJsb2JkYXRh",
        "captcha_rqtoken": "token123"
    });

    let body = serde_json::to_string(&captcha_response).unwrap();
    match CaptchaChallenge::from_response_body(&body) {
        CaptchaDetection::Challenge(c) => {
            assert_eq!(c.captcha_sitekey, "f5561ba9-8f1e-40ca-9b5b-a0b3f775f58e");
            assert_eq!(c.captcha_service, "hcaptcha");
            assert_eq!(c.captcha_rqdata.unwrap(), "dGVzdGJsb2JkYXRh");
            assert_eq!(c.captcha_rqtoken.unwrap(), "token123");
        }
        CaptchaDetection::NotCaptcha => panic!("Should have detected captcha"),
    }
}

// ==================== Moderation Tests ====================

#[tokio::test]
async fn test_kick_member() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/guilds/g1/members/u1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let result = client.kick_member("g1", "u1", Some("test reason")).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ban_member() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v10/guilds/g1/bans/u1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let result = client
        .ban_member("g1", "u1", Some("test ban"), Some(86400))
        .await;
    assert!(result.is_ok());
}

// ==================== Reaction Tests ====================

#[tokio::test]
async fn test_add_reaction() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path_regex(
            "/api/v10/channels/.*/messages/.*/reactions/.*/@me",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let result = client.add_reaction("ch1", "m1", "👍").await;
    assert!(result.is_ok());
}

// ==================== DM Tests ====================

#[tokio::test]
async fn test_get_dm_channels() {
    let server = MockServer::start().await;

    let dms = serde_json::json!([{
        "id": "dm1",
        "type": 1,
        "recipients": [mock_user_json("u1", "Friend")],
        "last_message_id": "m99"
    }]);

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me/channels"))
        .respond_with(discord_json_response(&dms))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let channels = client.get_dm_channels().await.unwrap();
    assert_eq!(channels.len(), 1);
    assert!(channels[0].is_dm());
}

// ==================== Delete Message Test ====================

#[tokio::test]
async fn test_delete_message() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v10/channels/ch1/messages/m1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let result = client.delete_message("ch1", "m1").await;
    assert!(result.is_ok());
}

// ==================== Fingerprint Header Tests ====================

#[tokio::test]
async fn test_requests_include_fingerprint_headers() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v10/users/@me"))
        .and(header("Origin", "https://discord.com"))
        .and(header("Referer", "https://discord.com/channels/@me"))
        .and(header_exists("X-Super-Properties"))
        .and(header_exists("X-Discord-Locale"))
        .respond_with(discord_json_response(&mock_user_json("123", "Test")))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let result = client.validate_token().await;
    assert!(
        result.is_ok(),
        "Request should include all fingerprint headers"
    );
}

// ==================== Invite Tests ====================

#[tokio::test]
async fn test_get_invite() {
    let server = MockServer::start().await;

    let invite = serde_json::json!({
        "code": "abc123",
        "guild": mock_guild_json("g1", "Test Guild"),
        "channel": {"id": "ch1", "name": "general", "type": 0},
        "approximate_member_count": 100,
        "approximate_presence_count": 50
    });

    Mock::given(method("GET"))
        .and(path_regex("/api/v10/invites/.*"))
        .respond_with(discord_json_response(&invite))
        .mount(&server)
        .await;

    let client = create_test_client(&server).await;
    client.set_token("token".to_string()).await;

    let invite = client.get_invite("abc123").await.unwrap();
    assert_eq!(invite.code, "abc123");
}

// ==================== Helper to create client pointing at mock server ====================

async fn create_test_client(server: &MockServer) -> discord_qt::client::DiscordClient {
    use discord_qt::client::DiscordClient;

    // Point the client at our mock server instead of discord.com
    // The mock server URL is like http://127.0.0.1:PORT
    // Client builds URLs as: {api_base}/v{version}{endpoint}
    // So mocks should match paths like /api/v10/users/@me
    let base_url = format!("{}/api", server.uri());
    let mut client = DiscordClient::new().await.unwrap();
    client.set_api_base(base_url);
    client
}
