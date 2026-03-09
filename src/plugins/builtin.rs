// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Built-in plugin manifests
//!
//! These plugins (message_logger, fake_mute, fake_deafen) are implemented
//! in Rust. Their manifests define options and metadata for the plugin UI.

use super::manifest::{
    ButtonPlacement, OptionCategory, PluginManifest, PluginOption, PluginOptionType,
    PluginUi, PluginUiButton, PluginUiModal, PluginUiModalField, ModalFieldType,
};

/// Get all built-in plugin manifests, sorted by priority (most requested first)
pub fn builtin_manifests() -> Vec<PluginManifest> {
    vec![
        // 1. Message Logger — track deleted/edited (Vencord, Vesktop, BD)
        message_logger_manifest(),
        // 2. Blur NSFW — blur attachments in NSFW channels until clicked
        blur_nsfw_manifest(),
        // 3. Show Hidden Channels — see channels you lack perms for
        show_hidden_channels_manifest(),
        // 4. Fake Mute / 5. Fake Deafen
        fake_mute_manifest(),
        fake_deafen_manifest(),
        // 6. Read All Notifications — bulk mark channels read
        read_all_notifications_manifest(),
        // 7. Clear URLs — strip tracking params
        clear_urls_manifest(),
        // 8. Silent Messages — suppress notifications toggle
        silent_messages_manifest(),
        // 9. Pin DMs — pin DMs to top
        pin_dms_manifest(),
        // 10. No Reply Mention — don't ping on reply
        no_reply_mention_manifest(),
        // 11. Image Zoom — click to zoom images
        image_zoom_manifest(),
        // 12. Always Animate — force animated avatars/emojis
        always_animate_manifest(),
        // 13. Custom RPC / arRPC
        custom_rpc_manifest(),
    ]
}

fn message_logger_manifest() -> PluginManifest {
    PluginManifest {
        id: "message-logger".to_string(),
        name: "Message Logger".to_string(),
        description: "Track deleted and edited messages locally with full edit history and search"
            .to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![
            PluginOption {
                key: "cache_size".to_string(),
                label: "Cache Size".to_string(),
                description: "Maximum number of messages to keep in memory".to_string(),
                option_type: PluginOptionType::Number,
                default: serde_json::json!(10000),
                category: OptionCategory::Storage,
                choices: None,
                min: Some(100.0),
                max: Some(50000.0),
            },
            PluginOption {
                key: "show_deleted_style".to_string(),
                label: "Deleted Message Style".to_string(),
                description: "How to display deleted messages".to_string(),
                option_type: PluginOptionType::Select,
                default: serde_json::json!("strikethrough"),
                category: OptionCategory::Display,
                choices: Some(vec![
                    "strikethrough".to_string(),
                    "faded".to_string(),
                    "deleted".to_string(),
                ]),
                min: None,
                max: None,
            },
        ],
        events: vec![
            "MESSAGE_CREATE".to_string(),
            "MESSAGE_UPDATE".to_string(),
            "MESSAGE_DELETE".to_string(),
            "MESSAGE_DELETE_BULK".to_string(),
        ],
        entry: "main.lua".to_string(),
        ui: PluginUi {
            buttons: vec![PluginUiButton {
                id: "export".to_string(),
                label: "Export Log".to_string(),
                tooltip: "Export message log to file".to_string(),
                placement: ButtonPlacement::Toolbar,
            }],
            modals: vec![PluginUiModal {
                id: "export_modal".to_string(),
                title: "Export Message Log".to_string(),
                fields: vec![
                    PluginUiModalField {
                        key: "format".to_string(),
                        label: "Format".to_string(),
                        field_type: ModalFieldType::Short,
                        placeholder: "json or csv".to_string(),
                        required: true,
                    },
                    PluginUiModalField {
                        key: "path".to_string(),
                        label: "Save path".to_string(),
                        field_type: ModalFieldType::Short,
                        placeholder: "/path/to/file".to_string(),
                        required: false,
                    },
                ],
            }],
        },
    }
}

fn fake_mute_manifest() -> PluginManifest {
    PluginManifest {
        id: "fake-mute".to_string(),
        name: "Fake Mute".to_string(),
        description: "Appear muted to others while still receiving audio".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![
            PluginOption {
                key: "record_audio".to_string(),
                label: "Record Audio".to_string(),
                description: "Record incoming audio while fake muted".to_string(),
                option_type: PluginOptionType::Boolean,
                default: serde_json::json!(false),
                category: OptionCategory::Voice,
                choices: None,
                min: None,
                max: None,
            },
            PluginOption {
                key: "send_silence".to_string(),
                label: "Send Silence Frames".to_string(),
                description: "Send silence to keep UDP connection alive".to_string(),
                option_type: PluginOptionType::Boolean,
                default: serde_json::json!(true),
                category: OptionCategory::Voice,
                choices: None,
                min: None,
                max: None,
            },
        ],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn fake_deafen_manifest() -> PluginManifest {
    PluginManifest {
        id: "fake-deafen".to_string(),
        name: "Fake Deafen".to_string(),
        description: "Appear deafened to others while still hearing everything".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![PluginOption {
            key: "record_audio".to_string(),
            label: "Record Audio".to_string(),
            description: "Record incoming audio while fake deafened".to_string(),
            option_type: PluginOptionType::Boolean,
            default: serde_json::json!(false),
            category: OptionCategory::Voice,
            choices: None,
            min: None,
            max: None,
        }],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn blur_nsfw_manifest() -> PluginManifest {
    PluginManifest {
        id: "blur-nsfw".to_string(),
        name: "Blur NSFW".to_string(),
        description: "Blur images and attachments in NSFW channels until you click to reveal"
            .to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![
            PluginOption {
                key: "blur_videos".to_string(),
                label: "Blur Videos".to_string(),
                description: "Also blur video thumbnails".to_string(),
                option_type: PluginOptionType::Boolean,
                default: serde_json::json!(true),
                category: OptionCategory::Display,
                choices: None,
                min: None,
                max: None,
            },
            PluginOption {
                key: "blur_level".to_string(),
                label: "Blur Strength".to_string(),
                description: "How strong the blur effect is".to_string(),
                option_type: PluginOptionType::Select,
                default: serde_json::json!("medium"),
                category: OptionCategory::Display,
                choices: Some(vec!["light".to_string(), "medium".to_string(), "heavy".to_string()]),
                min: None,
                max: None,
            },
        ],
        events: vec!["MESSAGE_CREATE".to_string()],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn show_hidden_channels_manifest() -> PluginManifest {
    PluginManifest {
        id: "show-hidden-channels".to_string(),
        name: "Show Hidden Channels".to_string(),
        description: "Display channels you don't have permission to view".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![PluginOption {
            key: "show_as_muted".to_string(),
            label: "Dim Hidden Channels".to_string(),
            description: "Visually dim channels you can't access".to_string(),
            option_type: PluginOptionType::Boolean,
            default: serde_json::json!(true),
            category: OptionCategory::Display,
            choices: None,
            min: None,
            max: None,
        }],
        events: vec!["CHANNEL_CREATE".to_string(), "CHANNEL_UPDATE".to_string()],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn read_all_notifications_manifest() -> PluginManifest {
    PluginManifest {
        id: "read-all-notifications".to_string(),
        name: "Read All Notifications".to_string(),
        description: "One-click button to mark all channels as read".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn clear_urls_manifest() -> PluginManifest {
    PluginManifest {
        id: "clear-urls".to_string(),
        name: "Clear URLs".to_string(),
        description: "Strip tracking parameters (utm_*, fbclid, etc.) from URLs before sending"
            .to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![PluginOption {
            key: "strip_on_send".to_string(),
            label: "Strip on Send".to_string(),
            description: "Automatically strip when sending messages".to_string(),
            option_type: PluginOptionType::Boolean,
            default: serde_json::json!(true),
            category: OptionCategory::Privacy,
            choices: None,
            min: None,
            max: None,
        }],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn silent_messages_manifest() -> PluginManifest {
    PluginManifest {
        id: "silent-messages".to_string(),
        name: "Silent Messages".to_string(),
        description: "Toggle to send messages without triggering notifications".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![PluginOption {
            key: "default_silent".to_string(),
            label: "Default to Silent".to_string(),
            description: "New messages start with silent mode on".to_string(),
            option_type: PluginOptionType::Boolean,
            default: serde_json::json!(false),
            category: OptionCategory::General,
            choices: None,
            min: None,
            max: None,
        }],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn pin_dms_manifest() -> PluginManifest {
    PluginManifest {
        id: "pin-dms".to_string(),
        name: "Pin DMs".to_string(),
        description: "Pin DM conversations to the top of the list".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn no_reply_mention_manifest() -> PluginManifest {
    PluginManifest {
        id: "no-reply-mention".to_string(),
        name: "No Reply Mention".to_string(),
        description: "Replies don't ping the original author by default".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn image_zoom_manifest() -> PluginManifest {
    PluginManifest {
        id: "image-zoom".to_string(),
        name: "Image Zoom".to_string(),
        description: "Click images to zoom in, scroll to pan".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![PluginOption {
            key: "zoom_level".to_string(),
            label: "Default Zoom".to_string(),
            description: "Initial zoom level when opening".to_string(),
            option_type: PluginOptionType::Select,
            default: serde_json::json!("fit"),
            category: OptionCategory::Display,
            choices: Some(vec!["fit".to_string(), "100%".to_string(), "150%".to_string()]),
            min: None,
            max: None,
        }],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn always_animate_manifest() -> PluginManifest {
    PluginManifest {
        id: "always-animate".to_string(),
        name: "Always Animate".to_string(),
        description: "Force animated avatars and emojis to always play".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

fn custom_rpc_manifest() -> PluginManifest {
    PluginManifest {
        id: "custom-rpc".to_string(),
        name: "Custom RPC / arRPC".to_string(),
        description: "Game activity detection and custom rich presence".to_string(),
        version: "1.0.0".to_string(),
        author: "Skunkcord".to_string(),
        repository: None,
        options: vec![PluginOption {
            key: "process_scan".to_string(),
            label: "Process Scanning".to_string(),
            description: "Auto-detect running games".to_string(),
            option_type: PluginOptionType::Boolean,
            default: serde_json::json!(true),
            category: OptionCategory::General,
            choices: None,
            min: None,
            max: None,
        }],
        events: vec![],
        entry: "main.lua".to_string(),
        ui: PluginUi::default(),
    }
}

