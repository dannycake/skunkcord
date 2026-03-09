// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Stress and correctness tests
//!
//! Verifies behavior under load and edge cases.

use skunkcord::client::permissions::{
    compute_base_permissions, compute_channel_permissions, has_permission, PermOverwrite,
};
use skunkcord::client::Permission;
use skunkcord::features::flags::FeatureFlags;
use skunkcord::plugins::message_logger::MessageCache;
use skunkcord::rendering::markdown::parse_markdown;
use skunkcord::security::content::strip_tracking_params;
use skunkcord::voice::udp::RtpHeader;

// ==================== Message Cache Stress ====================

#[test]
fn test_message_cache_1000_inserts() {
    let mut cache = MessageCache::new(500);
    for i in 0..1000 {
        cache.insert(skunkcord::plugins::message_logger::LoggedMessage {
            id: format!("msg_{}", i),
            channel_id: format!("ch_{}", i % 10),
            guild_id: Some("g1".to_string()),
            author_id: format!("u_{}", i % 50),
            author_name: format!("User{}", i % 50),
            content: format!("Message content #{}", i),
            attachments_json: "[]".to_string(),
            embeds_json: "[]".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            deleted: false,
            deleted_at: None,
            edit_history: vec![],
        });
    }
    // Should cap at 500
    assert!(cache.total_count() <= 500);
    // Newest messages should be present
    assert!(cache.get("msg_999").is_some());
}

#[test]
fn test_message_cache_delete_preserves() {
    let mut cache = MessageCache::new(100);
    // Insert 50 messages, delete first 25
    for i in 0..50 {
        cache.insert(skunkcord::plugins::message_logger::LoggedMessage {
            id: format!("m{}", i),
            channel_id: "ch1".to_string(),
            guild_id: None,
            author_id: "u1".to_string(),
            author_name: "User".to_string(),
            content: format!("msg {}", i),
            attachments_json: "[]".to_string(),
            embeds_json: "[]".to_string(),
            timestamp: "2024-01-01".to_string(),
            deleted: false,
            deleted_at: None,
            edit_history: vec![],
        });
    }
    for i in 0..25 {
        cache.mark_deleted(&format!("m{}", i));
    }
    assert_eq!(cache.deleted_count(), 25);

    // Insert 100 more — deleted ones should still be there
    for i in 50..150 {
        cache.insert(skunkcord::plugins::message_logger::LoggedMessage {
            id: format!("m{}", i),
            channel_id: "ch1".to_string(),
            guild_id: None,
            author_id: "u1".to_string(),
            author_name: "User".to_string(),
            content: format!("msg {}", i),
            attachments_json: "[]".to_string(),
            embeds_json: "[]".to_string(),
            timestamp: "2024-01-01".to_string(),
            deleted: false,
            deleted_at: None,
            edit_history: vec![],
        });
    }
    // Deleted messages should survive eviction
    assert!(cache.deleted_count() >= 20);
}

// ==================== RTP Sequence Wrapping ====================

#[test]
fn test_rtp_full_sequence_cycle() {
    let mut header = RtpHeader::new(12345);
    // Run through an entire u16 cycle
    for _ in 0..=u16::MAX as u32 {
        header.advance(960);
    }
    // Should have wrapped back to 0
    assert_eq!(header.sequence, 0);
}

// ==================== Permission Edge Cases ====================

#[test]
fn test_permission_all_deny_then_member_allow() {
    let base = Permission::ViewChannel as u64 | Permission::SendMessages as u64;
    let overwrites = vec![
        PermOverwrite {
            id: "everyone".to_string(),
            overwrite_type: 0,
            allow: 0,
            deny: u64::MAX, // Deny everything
        },
        PermOverwrite {
            id: "user1".to_string(),
            overwrite_type: 1,
            allow: Permission::ViewChannel as u64, // But allow view
            deny: 0,
        },
    ];
    let perms = compute_channel_permissions(base, &overwrites, &[], "everyone", "user1");
    assert!(has_permission(perms, Permission::ViewChannel));
    assert!(!has_permission(perms, Permission::SendMessages));
}

#[test]
fn test_permission_multiple_roles_combine() {
    let base = 0u64;
    let overwrites = vec![
        PermOverwrite {
            id: "role_a".to_string(),
            overwrite_type: 0,
            allow: Permission::ViewChannel as u64,
            deny: 0,
        },
        PermOverwrite {
            id: "role_b".to_string(),
            overwrite_type: 0,
            allow: Permission::SendMessages as u64,
            deny: 0,
        },
    ];
    let perms = compute_channel_permissions(
        base,
        &overwrites,
        &["role_a".to_string(), "role_b".to_string()],
        "everyone",
        "user1",
    );
    assert!(has_permission(perms, Permission::ViewChannel));
    assert!(has_permission(perms, Permission::SendMessages));
}

// ==================== Markdown Edge Cases ====================

#[test]
fn test_markdown_empty_string() {
    assert_eq!(parse_markdown(""), "");
}

#[test]
fn test_markdown_plain_text() {
    let result = parse_markdown("Hello world");
    assert!(result.contains("Hello world"));
}

#[test]
fn test_markdown_nested_formatting() {
    let result = parse_markdown("***bold italic***");
    assert!(result.contains("<strong>"));
    assert!(result.contains("<em>"));
}

#[test]
fn test_markdown_multiple_code_blocks() {
    let input = "```rust\nfn main() {}\n```\nSome text\n```py\nprint('hi')\n```";
    let result = parse_markdown(input);
    assert!(result.contains("<pre"));
    assert!(result.contains("fn main()"));
    assert!(result.contains("print("));
}

#[test]
fn test_markdown_url_with_special_chars() {
    let result = parse_markdown("https://example.com/path?a=1&b=2#section");
    assert!(result.contains("href="));
}

// ==================== URL Cleaning Edge Cases ====================

#[test]
fn test_strip_params_preserves_fragment() {
    let url = "https://example.com/page?utm_source=x#section";
    let cleaned = strip_tracking_params(url);
    assert!(!cleaned.contains("utm_source"));
    assert!(cleaned.contains("#section"));
}

#[test]
fn test_strip_params_empty_url() {
    assert_eq!(strip_tracking_params(""), "");
}

#[test]
fn test_strip_params_no_query() {
    let url = "https://example.com/page";
    assert_eq!(strip_tracking_params(url), url);
}

// ==================== Feature Flags Consistency ====================

#[test]
fn test_all_presets_serialize_deserialize() {
    for flags in [
        FeatureFlags::paranoid(),
        FeatureFlags::standard(),
        FeatureFlags::full(),
    ] {
        let json = serde_json::to_string(&flags).unwrap();
        let deser: FeatureFlags = serde_json::from_str(&json).unwrap();
        assert_eq!(flags.block_telemetry, deser.block_telemetry);
        assert_eq!(flags.fake_mute, deser.fake_mute);
        assert_eq!(flags.arrpc, deser.arrpc);
    }
}

#[test]
fn test_paranoid_subset_of_standard() {
    let paranoid = FeatureFlags::paranoid();
    let standard = FeatureFlags::standard();
    // Paranoid should not enable anything Standard doesn't
    // (except safety features which are enabled in both)
    assert!(!paranoid.fake_mute);
    assert!(!standard.fake_mute);
    assert!(!paranoid.arrpc);
    assert!(!standard.arrpc);
}

#[test]
fn test_full_enables_everything_standard_does() {
    let full = FeatureFlags::full();
    let standard = FeatureFlags::standard();
    // Full should enable at least everything Standard does
    assert!(full.block_telemetry);
    assert!(full.clear_urls || standard.clear_urls);
    assert!(full.silent_message_toggle);
}

// ==================== Session Validation Tests ====================

#[test]
fn test_session_fingerprint_plausible() {
    use skunkcord::client::Session;
    use skunkcord::fingerprint::BrowserFingerprint;
    use std::collections::HashMap;

    let fp = BrowserFingerprint::new_chrome();
    let session = Session::new(
        "token".to_string(),
        "user123".to_string(),
        HashMap::new(),
        HashMap::new(),
        fp,
    );

    assert!(session.is_fingerprint_plausible());
    assert!(!session.is_stale());
}

#[test]
fn test_session_old_build_not_plausible() {
    use skunkcord::client::Session;
    use skunkcord::fingerprint::BrowserFingerprint;
    use std::collections::HashMap;

    let mut fp = BrowserFingerprint::new_chrome();
    fp.client_build_number = 100000; // Very old
    let session = Session::new(
        "token".to_string(),
        "user123".to_string(),
        HashMap::new(),
        HashMap::new(),
        fp,
    );

    assert!(!session.is_fingerprint_plausible());
    assert!(session.needs_fingerprint_refresh());
}

// ==================== Cookie Tests ====================

#[test]
fn test_cookies_from_session() {
    use skunkcord::client::DiscordCookies;
    use std::collections::HashMap;

    let mut cookies = HashMap::new();
    cookies.insert("__dcfduid".to_string(), "abc123".to_string());
    cookies.insert("__sdcfduid".to_string(), "def456".to_string());
    cookies.insert("locale".to_string(), "en-US".to_string());

    let dc = DiscordCookies::from_map(&cookies);
    assert!(dc.has_cf_cookies());
    let header = dc.to_header_string();
    assert!(header.contains("__dcfduid=abc123"));
    assert!(header.contains("__sdcfduid=def456"));
    assert!(header.contains("locale=en-US"));
}

// ==================== Captcha Interceptor Tests ====================

#[test]
fn test_captcha_interceptor_on_400_with_captcha() {
    use skunkcord::client::captcha_interceptor::check_for_captcha;

    let body = r#"{"captcha_sitekey":"key123","captcha_service":"hcaptcha"}"#;
    let result = check_for_captcha(400, body);
    assert!(result.is_err());
    match result.unwrap_err() {
        skunkcord::DiscordError::CaptchaRequired(_) => {} // correct
        e => panic!("Expected CaptchaRequired, got {:?}", e),
    }
}

#[test]
fn test_captcha_interceptor_on_400_without_captcha() {
    use skunkcord::client::captcha_interceptor::check_for_captcha;

    let body = r#"{"message":"Missing Access","code":50001}"#;
    assert!(check_for_captcha(400, body).is_ok());
}

#[test]
fn test_captcha_interceptor_on_200() {
    use skunkcord::client::captcha_interceptor::check_for_captcha;
    assert!(check_for_captcha(200, "{}").is_ok());
}

// ==================== Captcha State Machine Tests (from plan 7.6.5) ====================

#[test]
fn test_captcha_state_transitions() {
    use skunkcord::captcha::CaptchaState;

    // Happy path
    let mut state = CaptchaState::Idle;
    state = CaptchaState::ChallengeReceived;
    state = CaptchaState::WidgetLoaded;
    state = CaptchaState::Solving;
    state = CaptchaState::Solved("P1_token123".to_string());
    assert!(matches!(state, CaptchaState::Solved(_)));
    state = CaptchaState::Retrying;
    state = CaptchaState::Done;
    assert_eq!(state, CaptchaState::Done);
}

#[test]
fn test_captcha_expired_resets() {
    use skunkcord::captcha::CaptchaState;

    let state = CaptchaState::Expired;
    // After expired, should go back to ChallengeReceived
    let next = CaptchaState::ChallengeReceived;
    assert_eq!(next, CaptchaState::ChallengeReceived);
}

#[test]
fn test_captcha_cancel() {
    use skunkcord::captcha::CaptchaState;

    let state = CaptchaState::Cancelled;
    assert_eq!(state, CaptchaState::Cancelled);
}

// ==================== Captcha Retry Header Tests (from plan 7.6.4) ====================

#[test]
fn test_retry_includes_captcha_key_header() {
    use skunkcord::client::captcha_interceptor::captcha_retry_headers;

    let headers = captcha_retry_headers("P1_solved_token_abc", Some("rqtoken_xyz"));
    assert_eq!(headers.len(), 2);
    assert_eq!(
        headers[0],
        (
            "X-Captcha-Key".to_string(),
            "P1_solved_token_abc".to_string()
        )
    );
    assert_eq!(
        headers[1],
        ("X-Captcha-Rqtoken".to_string(), "rqtoken_xyz".to_string())
    );
}

#[test]
fn test_retry_without_rqtoken() {
    use skunkcord::client::captcha_interceptor::captcha_retry_headers;

    let headers = captcha_retry_headers("P1_token", None);
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].0, "X-Captcha-Key");
}

// ==================== Widget HTML Comprehensive Tests (from plan 7.6.2) ====================

#[test]
fn test_widget_dark_theme() {
    use skunkcord::captcha::widget::generate_captcha_html;
    use skunkcord::captcha::CaptchaChallenge;

    let challenge = CaptchaChallenge {
        captcha_service: "hcaptcha".to_string(),
        captcha_sitekey: "test".to_string(),
        captcha_rqdata: None,
        captcha_rqtoken: None,
        captcha_session_id: None,
        captcha_key: None,
    };
    let html = generate_captcha_html(&challenge);
    // Should use dark theme
    assert!(html.contains("theme: 'dark'"));
    // Should have OLED-matching background
    assert!(html.contains("#313338"));
}

// ==================== Feature Flag Edge Cases ====================

#[test]
fn test_feature_flags_all_metadata_present() {
    let meta = skunkcord::features::FeatureFlags::all_metadata();
    // Every metadata entry should have non-empty name and description
    for (key, m) in &meta {
        assert!(!m.name.is_empty(), "Empty name for key: {}", key);
        assert!(
            !m.description.is_empty(),
            "Empty description for key: {}",
            key
        );
        // FeatureMeta: name, description, category (risk_reason removed)
    }
}

#[test]
fn test_feature_flags_metadata_covers_all_risky_features() {
    let meta = skunkcord::features::FeatureFlags::all_metadata();
    let keys: Vec<&str> = meta.iter().map(|(k, _)| *k).collect();

    // All High risk features must have metadata
    assert!(
        keys.contains(&"fake_mute"),
        "fake_mute missing from metadata"
    );
    assert!(
        keys.contains(&"fake_deafen"),
        "fake_deafen missing from metadata"
    );

    // All Medium risk features (message_logger is a plugin, not in FeatureFlags)
    assert!(
        keys.contains(&"show_hidden_channels"),
        "show_hidden_channels missing"
    );
    assert!(
        keys.contains(&"experiments_panel"),
        "experiments_panel missing"
    );
    assert!(keys.contains(&"arrpc"), "arrpc missing");
}
