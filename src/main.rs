// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Qt Client
//!
//! A user-account Discord client built with Rust and Qt, featuring browser
//! emulation for undetectable API access. OLED-black UI with JetBrains Mono.
//!
//! Usage:
//!   DISCORD_TOKEN=token ./discord_qt           # Use env var
//!   ./discord_qt                               # Interactive token prompt
//!   ./discord_qt --token <token>               # CLI argument

use discord_qt::app_runner::run_app_with_updates;
use discord_qt::bridge::{BackendBridge, LoginRequest, UiAction, UiUpdate};
use discord_qt::client::captcha_interceptor;
use discord_qt::client::DiscordClient;
use discord_qt::client::x_fingerprint::XFingerprintManager;
use discord_qt::features::FeatureFlags;
use discord_qt::fingerprint::BrowserFingerprint;
use discord_qt::gateway::{Gateway, GatewayEvent};
use discord_qt::storage::{AppSettings, Storage};
use discord_qt::ui::AppController;
use discord_qt::Result;
use qmetaobject::prelude::*;
use qmetaobject::QObjectPinned;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("discord_qt=info".parse().unwrap()),
        )
        .init();

    println!("╔══════════════════════════════════════════╗");
    println!(
        "║     Discord Qt Client v{}          ║",
        env!("CARGO_PKG_VERSION")
    );
    println!("║     OLED · JetBrains Mono · Rust         ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // Load settings
    let storage = Storage::new()?;
    let settings = storage.load_settings().unwrap_or_default();
    let feature_flags = settings.feature_flags.clone();

    // Channels: main thread <-> worker thread
    let (login_tx, login_rx) = mpsc::channel::<LoginRequest>();
    let (update_tx, update_rx) = mpsc::channel::<UiUpdate>();
    // Action channel: QML thread -> async worker (tokio unbounded — non-async send)
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<UiAction>();

    // Initial login request: only from CLI or env so the worker doesn't block on a saved-session
    // token while the user tries to log in with credentials (which would never be read).
    let initial_request = get_initial_login_request().map(LoginRequest::Token);

    // Shared receiver so credential login can wait for CaptchaSolution/MfaCode from the same channel
    let login_rx = Arc::new(Mutex::new(login_rx));

    // Spawn worker thread for async backend
    let storage_clone = storage.clone();
    let settings_clone = settings.clone();
    let login_rx_worker = Arc::clone(&login_rx);
    thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = update_tx.send(UiUpdate::LoginFailed(e.to_string()));
                return;
            }
        };

        rt.block_on(async {
            let action_rx = Arc::new(tokio::sync::Mutex::new(action_rx));
            let mut next_request = initial_request;

            loop {
                let req = if let Some(r) = next_request.take() {
                    r
                } else {
                    let rx = login_rx_worker.lock().unwrap();
                    match rx.recv() {
                        Ok(r) => r,
                        Err(_) => break,
                    }
                };

                let token = match resolve_login_request(
                    req,
                    Arc::clone(&login_rx_worker),
                    &update_tx,
                    &storage_clone,
                    &settings_clone,
                )
                .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = update_tx.send(UiUpdate::LoginFailed(e.to_string()));
                        continue;
                    }
                };

                if let Err(e) = run_app_with_updates(
                    token,
                    storage_clone.clone(),
                    settings_clone.clone(),
                    feature_flags.clone(),
                    update_tx.clone(),
                    Arc::clone(&action_rx),
                )
                .await
                {
                    let _ = update_tx.send(UiUpdate::LoginFailed(e.to_string()));
                }
            }
        });
    });

    // Create AppController and pass to QML
    let app_controller = std::cell::RefCell::new(AppController::new(login_tx, action_tx, update_rx));
    let controller_ptr = unsafe { QObjectPinned::new(&app_controller) };

    // Load and run Qt UI
    let mut engine = QmlEngine::new();
    engine.set_object_property("app".into(), controller_ptr);

    // Load QML from path relative to executable for deployment
    let qml_path = get_qml_path("main.qml");
    if !qml_path.exists() {
        eprintln!("ERROR: QML file not found at: {}", qml_path.display());
        eprintln!("Please ensure the 'qml' directory is in the same location as the executable.");
        std::process::exit(1);
    }
    engine.load_file(qml_path.to_string_lossy().to_string().into());
    engine.exec();
    Ok(())
}

/// Get a Discord token from various sources
fn get_token(settings: &AppSettings, storage: &Storage) -> Result<String> {
    // 1. Check CLI args
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--token" && i + 1 < args.len() {
            println!("  Token: from --token argument");
            return Ok(args[i + 1].clone());
        }
    }

    // 2. Check environment variable
    if let Ok(token) = std::env::var("DISCORD_TOKEN") {
        println!("  Token: from DISCORD_TOKEN env var");
        return Ok(token);
    }

    // 3. Check saved session
    if let Some(ref last_id) = settings.last_account_id {
        if let Ok(Some(session)) = storage.load_session(last_id) {
            if !session.is_stale() {
                if session.needs_fingerprint_refresh() {
                    println!(
                        "  Token: from saved session (user {}) [fingerprint will be refreshed]",
                        last_id
                    );
                } else {
                    println!("  Token: from saved session (user {})", last_id);
                }
                return Ok(session.token);
            } else {
                println!(
                    "  Saved session for {} is stale (>7 days), skipping",
                    last_id
                );
            }
        }
    }

    // 4. Interactive prompt
    println!();
    println!("  ┌─────────────────────────────────────┐");
    println!("  │         Enter Discord Token          │");
    println!("  │                                      │");
    println!("  │  Your token is stored locally and    │");
    println!("  │  never sent anywhere except Discord. │");
    println!("  └─────────────────────────────────────┘");
    println!();
    print!("  Token: ");
    io::stdout().flush().unwrap();

    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    let token = token.trim().to_string();

    if token.is_empty() {
        return Ok(String::new());
    }

    // Strip quotes if user pasted with them
    let token = token.trim_matches('"').trim_matches('\'').to_string();

    Ok(token)
}

/// Run the application with a valid token
async fn run_app(
    token: String,
    storage: Storage,
    settings: AppSettings,
    flags: FeatureFlags,
) -> Result<()> {
    // Create fingerprint
    let fingerprint = BrowserFingerprint::new_chrome();
    println!(
        "  Browser: Chrome {} (build {})",
        fingerprint.browser_version, fingerprint.client_build_number
    );

    if flags.block_telemetry {
        println!("  Telemetry: BLOCKED (/science, /track, /metrics)");
    }

    // Create Discord client
    let client = Arc::new(DiscordClient::with_fingerprint(fingerprint.clone()).await?);
    client.set_token(token.clone()).await;

    // Validate token
    print!("  Validating token... ");
    io::stdout().flush().unwrap();

    match client.validate_token().await {
        Ok(user) => {
            println!("✓");
            println!();
            println!("  ┌─────────────────────────────────────┐");
            println!(
                "  │  Logged in as: {:<21}│",
                format!("{}#{}", user.username, user.discriminator)
            );
            if let Some(ref gn) = user.global_name {
                println!("  │  Display name: {:<20}│", gn);
            }
            println!("  │  User ID: {:<25}│", user.id);
            println!("  └─────────────────────────────────────┘");

            // Save session
            let session = discord_qt::client::Session::new(
                token.clone(),
                user.id.clone(),
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
                fingerprint.clone(),
            );
            let _ = storage.save_session(&session);
            let mut new_settings = settings.clone();
            new_settings.last_account_id = Some(user.id.clone());
            let _ = storage.save_settings(&new_settings);

            // Fetch guilds
            println!();
            match client.get_guilds().await {
                Ok(guilds) => {
                    println!("  Guilds ({}):", guilds.len());
                    for (i, guild) in guilds.iter().enumerate().take(15) {
                        println!("    {}. {}", i + 1, guild.name);
                    }
                    if guilds.len() > 15 {
                        println!("    ... and {} more", guilds.len() - 15);
                    }
                }
                Err(e) => println!("  Failed to fetch guilds: {}", e),
            }

            // Fetch DMs
            match client.get_dm_channels().await {
                Ok(dms) => {
                    println!("  DM channels: {}", dms.len());
                }
                Err(e) => println!("  Failed to fetch DMs: {}", e),
            }

            // Fetch relationships
            match client.get_relationships().await {
                Ok(rels) => {
                    let friends = rels.iter().filter(|r| r.is_friend()).count();
                    let blocked = rels.iter().filter(|r| r.is_blocked()).count();
                    let pending = rels
                        .iter()
                        .filter(|r| r.is_incoming_request() || r.is_outgoing_request())
                        .count();
                    println!(
                        "  Friends: {}, Blocked: {}, Pending: {}",
                        friends, blocked, pending
                    );
                }
                Err(e) => println!("  Failed to fetch relationships: {}", e),
            }

            // Create backend bridge (CLI path: no plugins, no hooks)
            let (bridge, _action_tx, _update_rx) = BackendBridge::new(
                Arc::clone(&client),
                flags.clone(),
                std::collections::HashMap::new(),
                None,
            );

            // Connect to gateway
            println!();
            println!("  Connecting to gateway (auto-reconnect enabled)...");
            println!("  Press Ctrl+C to disconnect.");
            println!();

            let mut gateway = Gateway::new(token, fingerprint);
            let mut events = gateway.subscribe();

            // Spawn event handler
            tokio::spawn(async move {
                while let Ok(event) = events.recv().await {
                    bridge.handle_gateway_event(&event).await;

                    match &event {
                        GatewayEvent::Ready(ready) => {
                            println!(
                                "  [Gateway] Ready! Session: {}, {} guilds",
                                &ready.session_id[..8],
                                ready.guilds.len()
                            );
                        }
                        GatewayEvent::MessageCreate(msg) => {
                            if let Some(author) = &msg.message.author {
                                let content: String =
                                    msg.message.content.chars().take(80).collect();
                                println!("  [Message] {}: {}", author.display_name(), content);
                            }
                        }
                        GatewayEvent::PresenceUpdate(p) => {
                            tracing::debug!("Presence: {} → {}", p.user.id, p.status);
                        }
                        GatewayEvent::MessageDelete(d) => {
                            println!("  [Deleted] Message {} in channel {}", d.id, d.channel_id);
                        }
                        GatewayEvent::TypingStart(t) => {
                            tracing::debug!("Typing: {} in {}", t.user_id, t.channel_id);
                        }
                        GatewayEvent::VoiceStateUpdate(v) => {
                            println!("  [Voice] User {} → channel {:?}", v.user_id, v.channel_id);
                        }
                        _ => {}
                    }
                }
            });

            // Connect with auto-reconnect (blocks until disconnect)
            if let Err(e) = gateway.connect_with_reconnect().await {
                println!("  [Gateway] Error: {}", e);
            }

            println!("  Disconnected.");
        }
        Err(e) => {
            println!("✗");
            println!("  Token validation failed: {}", e);
            println!("  Please check your token and try again.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use discord_qt::fingerprint::BrowserFingerprint;

    #[tokio::test]
    async fn test_client_creation() {
        let fingerprint = BrowserFingerprint::new_chrome();
        let client = discord_qt::client::DiscordClient::with_fingerprint(fingerprint).await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_fingerprint_generation() {
        let fp = BrowserFingerprint::new_chrome();
        assert!(!fp.user_agent.is_empty());
        assert!(!fp.x_super_properties.is_empty());
        assert!(fp.client_build_number > 0);
    }
}

/// Resolve a login request to a token. For credential login, may block (via spawn_blocking)
/// waiting for CaptchaSolution or MfaCode from the login channel.
async fn resolve_login_request(
    req: LoginRequest,
    login_rx: Arc<Mutex<mpsc::Receiver<LoginRequest>>>,
    update_tx: &mpsc::Sender<UiUpdate>,
    storage: &Storage,
    settings: &AppSettings,
) -> Result<String> {
    match req {
        LoginRequest::Token(t) => return Ok(t),
        LoginRequest::SwitchAccount(account_id) => {
            let session = storage
                .load_session(&account_id)?
                .ok_or_else(|| discord_qt::DiscordError::Http("Session not found".to_string()))?;
            if session.is_stale() {
                return Err(discord_qt::DiscordError::Http(
                    "Session expired".to_string(),
                )
                .into());
            }
            return Ok(session.token);
        }
        LoginRequest::Credentials { email, password } => {
            let fingerprint = BrowserFingerprint::new_chrome();
            let x_fp = XFingerprintManager::new();
            let x_fingerprint = x_fp.get_or_fetch().await?;

            let mut captcha_key: Option<String> = None;
            let mut captcha_rqtoken: Option<String> = None;
            let mut captcha_session_id: Option<String> = None;

            #[cfg(feature = "wreq-auth")]
            {
                // Use wreq (Chrome TLS/HTTP2 fingerprint) for login and MFA to avoid Discord/Cloudflare antibot
                let wreq_client = discord_qt::client::wreq_auth::build_wreq_client()?;
                loop {
                    let resp = discord_qt::client::wreq_auth::login_with_credentials_wreq(
                        &wreq_client,
                        &fingerprint,
                        &email,
                        &password,
                        &x_fingerprint,
                        captcha_key.as_deref(),
                        captcha_rqtoken.as_deref(),
                        captcha_session_id.as_deref(),
                    )
                    .await;

                    match resp {
                        Ok(login_resp) => {
                            if let Some(token) = login_resp.token {
                                return Ok(token);
                            }
                            if login_resp.mfa == Some(true) {
                                let ticket = login_resp
                                    .ticket
                                    .clone()
                                    .ok_or_else(|| discord_qt::DiscordError::Http("MFA required but no ticket".to_string()))?;
                                let login_instance_id = login_resp.login_instance_id.clone();
                                let _ = update_tx.send(UiUpdate::MfaRequired {
                                    ticket: ticket.clone(),
                                    login_instance_id: login_instance_id.clone(),
                                    sms: login_resp.sms.unwrap_or(false),
                                    totp: login_resp.totp.unwrap_or(true),
                                    backup: login_resp.backup.unwrap_or(false),
                                });
                                let login_rx_clone = Arc::clone(&login_rx);
                                let next = tokio::task::spawn_blocking(move || {
                                    login_rx_clone.lock().unwrap().recv().ok()
                                })
                                .await
                                .ok()
                                .flatten();
                                match next {
                                    Some(LoginRequest::MfaCode {
                                        ticket: t,
                                        code,
                                        login_instance_id: inst_id,
                                    }) => {
                                        let mfa_resp = discord_qt::client::wreq_auth::verify_mfa_totp_wreq(
                                            &wreq_client,
                                            &fingerprint,
                                            &t,
                                            &code,
                                            &x_fingerprint,
                                            inst_id.as_deref(),
                                        )
                                        .await?;
                                        return Ok(mfa_resp.token);
                                    }
                                    Some(LoginRequest::CancelMfa) => {
                                        return Err(discord_qt::DiscordError::Http(
                                            "Login cancelled".to_string(),
                                        )
                                        .into());
                                    }
                                    _ => {
                                        return Err(discord_qt::DiscordError::Http(
                                            "MFA required but no MfaCode received".to_string(),
                                        )
                                        .into());
                                    }
                                }
                            }
                            return Err(discord_qt::DiscordError::Http(
                                "Login response had no token and no MFA".to_string(),
                            )
                            .into());
                        }
                        Err(e) => {
                            if let discord_qt::DiscordError::CaptchaRequired(_) = &e {
                                let challenge = captcha_interceptor::extract_challenge(&e);
                                if let Some(c) = challenge {
                                    let _ = update_tx.send(UiUpdate::CaptchaRequired {
                                        sitekey: c.captcha_sitekey,
                                        rqdata: c.captcha_rqdata,
                                        rqtoken: c.captcha_rqtoken.clone(),
                                        captcha_session_id: c.captcha_session_id.clone(),
                                    });
                                    let login_rx_clone = Arc::clone(&login_rx);
                                    let next = tokio::task::spawn_blocking(move || {
                                        login_rx_clone.lock().unwrap().recv().ok()
                                    })
                                    .await
                                    .ok()
                                    .flatten();
                                    if let Some(LoginRequest::CaptchaSolution {
                                        captcha_key: key,
                                        rqtoken: rqt,
                                    }) = next
                                    {
                                        captcha_key = Some(key);
                                        captcha_rqtoken = rqt;
                                        captcha_session_id = c.captcha_session_id.clone();
                                        continue;
                                    }
                                }
                            }
                            return Err(e.into());
                        }
                    }
                }
            }

            #[cfg(not(feature = "wreq-auth"))]
            {
                let proxy_config = settings.proxy_settings.to_proxy_config();
                let client = match &proxy_config {
                    Some(proxy) => Arc::new(
                        DiscordClient::with_proxy(fingerprint.clone(), proxy.clone()).await?,
                    ),
                    None => Arc::new(DiscordClient::with_fingerprint(fingerprint.clone()).await?),
                };
                loop {
                    let resp = client
                        .login_with_credentials(
                            &email,
                            &password,
                            &x_fingerprint,
                            captcha_key.as_deref(),
                            captcha_rqtoken.as_deref(),
                            captcha_session_id.as_deref(),
                        )
                        .await;

                    match resp {
                        Ok(login_resp) => {
                            if let Some(token) = login_resp.token {
                                return Ok(token);
                            }
                            if login_resp.mfa == Some(true) {
                                let ticket = login_resp
                                    .ticket
                                    .clone()
                                    .ok_or_else(|| discord_qt::DiscordError::Http("MFA required but no ticket".to_string()))?;
                                let login_instance_id = login_resp.login_instance_id.clone();
                                let _ = update_tx.send(UiUpdate::MfaRequired {
                                    ticket: ticket.clone(),
                                    login_instance_id: login_instance_id.clone(),
                                    sms: login_resp.sms.unwrap_or(false),
                                    totp: login_resp.totp.unwrap_or(true),
                                    backup: login_resp.backup.unwrap_or(false),
                                });
                                let login_rx_clone = Arc::clone(&login_rx);
                                let next = tokio::task::spawn_blocking(move || {
                                    login_rx_clone.lock().unwrap().recv().ok()
                                })
                                .await
                                .ok()
                                .flatten();
                                match next {
                                    Some(LoginRequest::MfaCode {
                                        ticket: t,
                                        code,
                                        login_instance_id: inst_id,
                                    }) => {
                                        let mfa_resp = client
                                            .verify_mfa_totp(&t, &code, &x_fingerprint, inst_id.as_deref())
                                            .await?;
                                        return Ok(mfa_resp.token);
                                    }
                                    Some(LoginRequest::CancelMfa) => {
                                        return Err(discord_qt::DiscordError::Http(
                                            "Login cancelled".to_string(),
                                        )
                                        .into());
                                    }
                                    _ => {
                                        return Err(discord_qt::DiscordError::Http(
                                            "MFA required but no MfaCode received".to_string(),
                                        )
                                        .into());
                                    }
                                }
                            }
                            return Err(discord_qt::DiscordError::Http(
                                "Login response had no token and no MFA".to_string(),
                            )
                            .into());
                        }
                        Err(e) => {
                            if let discord_qt::DiscordError::CaptchaRequired(_) = &e {
                                let challenge = captcha_interceptor::extract_challenge(&e);
                                if let Some(c) = challenge {
                                    let _ = update_tx.send(UiUpdate::CaptchaRequired {
                                        sitekey: c.captcha_sitekey,
                                        rqdata: c.captcha_rqdata,
                                        rqtoken: c.captcha_rqtoken.clone(),
                                        captcha_session_id: c.captcha_session_id.clone(),
                                    });
                                    let login_rx_clone = Arc::clone(&login_rx);
                                    let next = tokio::task::spawn_blocking(move || {
                                        login_rx_clone.lock().unwrap().recv().ok()
                                    })
                                    .await
                                    .ok()
                                    .flatten();
                                    if let Some(LoginRequest::CaptchaSolution {
                                        captcha_key: key,
                                        rqtoken: rqt,
                                    }) = next
                                    {
                                        captcha_key = Some(key);
                                        captcha_rqtoken = rqt;
                                        captcha_session_id = c.captcha_session_id.clone();
                                        continue;
                                    }
                                }
                            }
                            return Err(e.into());
                        }
                    }
                }
            }
        }
        LoginRequest::CaptchaSolution { .. } | LoginRequest::MfaCode { .. } | LoginRequest::CancelMfa => {
            Err(discord_qt::DiscordError::Http(
                "CaptchaSolution/MfaCode/CancelMfa must follow Credentials flow".to_string(),
            )
            .into())
        }
    }
}

/// Get token from CLI, env, or saved session only (no interactive prompt)
/// Token for initial worker request only when from CLI or env (not saved session).
/// This avoids the worker consuming a saved-session token and blocking in run_app_with_updates
/// while the user tries to log in with credentials (which would never be read from the channel).
fn get_initial_login_request() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--token" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    if let Ok(token) = std::env::var("DISCORD_TOKEN") {
        return Some(token);
    }
    None
}

fn get_token_no_prompt(settings: &AppSettings, storage: &Storage) -> Result<Option<String>> {
    if let Some(t) = get_initial_login_request() {
        return Ok(Some(t));
    }
    if let Some(ref last_id) = settings.last_account_id {
        if let Ok(Some(session)) = storage.load_session(last_id) {
            if !session.is_stale() {
                return Ok(Some(session.token));
            }
        }
    }
    Ok(None)
}

/// Get QML file path - tries multiple locations in order:
/// 1. app-qml/ (bundled package with Qt libs)
/// 2. qml/ (simple deployment package)
/// 3. Development path (CARGO_MANIFEST_DIR/src/qml)
fn get_qml_path(filename: &str) -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let bundled_path = exe_dir.join("app-qml").join(filename);
            if bundled_path.exists() {
                return bundled_path;
            }
            let qml_path = exe_dir.join("qml").join(filename);
            if qml_path.exists() {
                return qml_path;
            }
        }
    }
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("qml")
        .join(filename)
}
