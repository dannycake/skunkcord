// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Backend runner for the Discord client.
//!
//! Runs the async backend (gateway, bridge, action handler) with a given token
//! and sends UI updates to the provided channel. Used by both the desktop binary
//! (main.rs) and the mobile FFI layer (mobile_ffi.rs).

use crate::bridge::{handle_ui_action, BackendBridge, SharedBridgeCache, SharedGatewayCmd, UiAction, UiUpdate};
use crate::plugins;
use crate::client::{DiscordClient, Session};
use crate::features::FeatureFlags;
use crate::fingerprint::BrowserFingerprint;
use crate::gateway::Gateway;
use crate::storage::{AppSettings, Storage};
use crate::Result;
use std::sync::mpsc;
use std::sync::Arc;

/// Run the app with a token, sending UI updates to the channel.
/// Used by desktop (main.rs) and mobile (mobile_ffi.rs).
pub async fn run_app_with_updates(
    token: String,
    storage: Storage,
    settings: AppSettings,
    flags: FeatureFlags,
    update_tx: mpsc::Sender<UiUpdate>,
    action_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<UiAction>>>,
) -> Result<()> {
    let fingerprint = BrowserFingerprint::new_chrome();
    let proxy_config = settings.proxy_settings.to_proxy_config();
    let client = match &proxy_config {
        Some(proxy) => {
            tracing::info!("Creating client with proxy: {}:{}", proxy.host, proxy.port);
            Arc::new(DiscordClient::with_proxy(fingerprint.clone(), proxy.clone()).await?)
        }
        None => {
            tracing::info!("Creating client without proxy");
            Arc::new(DiscordClient::with_fingerprint(fingerprint.clone()).await?)
        }
    };
    client.set_token(token.clone()).await;

    match client.validate_token().await {
        Ok(user) => {
            let _ = update_tx.send(UiUpdate::LoginSuccess {
                user_id: user.id.clone(),
                username: user.display_name().to_string(),
                avatar_url: Some(user.avatar_url(128)),
            });

            let mut session = Session::new(
                token.clone(),
                user.id.clone(),
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
                fingerprint.clone(),
            );
            session.username = Some(user.display_name().to_string());
            session.avatar_url = Some(user.avatar_url(128));
            let _ = storage.save_session(&session);
            let mut new_settings = settings.clone();
            new_settings.last_account_id = Some(user.id.clone());
            let _ = storage.save_settings(&new_settings);

            // Send list of saved accounts for account switcher UI (user_id, display_name)
            if let Ok(sessions) = storage.load_all_sessions() {
                let accounts: Vec<(String, String)> = sessions
                    .values()
                    .map(|s| {
                        let name = s
                            .username
                            .as_deref()
                            .unwrap_or(&s.user_id)
                            .to_string();
                        (s.user_id.clone(), name)
                    })
                    .collect();
                let _ = update_tx.send(UiUpdate::AccountsList(accounts));
            }

            // Migrate: if plugin_enabled empty, derive from feature flags
            let mut plugin_enabled = settings.plugin_enabled.clone();
            if plugin_enabled.is_empty() {
                plugin_enabled.insert("message-logger".to_string(), false); // Plugin, not flag
                plugin_enabled.insert("fake-mute".to_string(), flags.fake_mute);
                plugin_enabled.insert("fake-deafen".to_string(), flags.fake_deafen);
            }
            let (gateway_hooks, message_logger_cache) = plugins::create_gateway_hooks(&plugin_enabled);
            let (bridge, _bridge_action_tx, mut bridge_update_rx) =
                BackendBridge::new(Arc::clone(&client), flags.clone(), plugin_enabled.clone(), gateway_hooks);

            // Sync plugin UI for already-enabled plugins
            let plugin_manifests = plugins::all_manifests();
            for (pid, enabled) in &plugin_enabled {
                if *enabled {
                    if let Some(manifest) = plugin_manifests.get(pid) {
                        if !manifest.ui.buttons.is_empty() || !manifest.ui.modals.is_empty() {
                            let _ = update_tx.send(UiUpdate::PluginUiUpdated {
                                plugin_id: pid.clone(),
                                buttons: manifest.ui.buttons.clone(),
                                modals: manifest.ui.modals.clone(),
                            });
                        }
                    }
                }
            }

            let update_tx_clone = update_tx.clone();
            tokio::spawn(async move {
                while let Ok(update) = bridge_update_rx.recv().await {
                    let _ = update_tx_clone.send(update);
                }
            });

            let mut gateway = Gateway::new(token, fingerprint, proxy_config);
            let mut events = gateway.subscribe();

            let gateway_cmd: SharedGatewayCmd = Arc::clone(&gateway.shared_cmd_tx);
            let gateway_proxy = Arc::clone(&gateway.proxy_config);
            let bridge_cache: SharedBridgeCache = Arc::clone(&bridge.bridge_cache);
            let bridge_flags = Arc::clone(&bridge.flags);
            let bridge_plugin_enabled = Arc::clone(&bridge.plugin_enabled);

            let client_for_actions = Arc::clone(&client);
            let update_tx_for_actions = update_tx.clone();
            let action_rx_ref = Arc::clone(&action_rx);
            let storage_for_actions = storage.clone();
            let plugin_manifests = Arc::new(tokio::sync::RwLock::new(plugins::all_manifests()));
            let action_handle = tokio::spawn(async move {
                let mut rx = action_rx_ref.lock().await;
                while let Some(action) = rx.recv().await {
                    handle_ui_action(
                        action,
                        &client_for_actions,
                        &update_tx_for_actions,
                        &gateway_cmd,
                        &bridge_cache,
                        &bridge_flags,
                        &bridge_plugin_enabled,
                        &plugin_manifests,
                        message_logger_cache.as_ref(),
                        &storage_for_actions,
                        Some(&gateway_proxy),
                    )
                    .await;
                }
            });

            tokio::spawn(async move {
                while let Ok(event) = events.recv().await {
                    bridge.handle_gateway_event(&event).await;
                }
            });

            // Spawn browser handoff RPC server (listens on localhost for invite
            // links beamed from the browser's "Open in App" flow)
            let handoff_tx = update_tx.clone();
            tokio::spawn(async move {
                crate::features::browser_handoff::run_browser_handoff_server(handoff_tx).await;
            });

            gateway.connect_with_reconnect().await?;

            action_handle.abort();
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}
