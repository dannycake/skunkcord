// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Qt UI Module
//!
//! Main Qt-based user interface for the Discord client.

pub mod app_controller;
pub mod login_window;
pub mod main_window;
pub mod styles;
pub mod web_view;

pub use app_controller::*;
pub use login_window::*;
pub use main_window::*;
pub use styles::*;
pub use web_view::*;

use qmetaobject::prelude::*;
use qmetaobject::qml_register_type;

/// Initialize the Qt application
pub fn init_qt_app() {
    // Register custom types with Qt meta-object system
    qml_register_type::<DiscordQtApp>(c"DiscordQt", 1, 0, c"DiscordQtApp");
}

/// Main Qt application controller
#[derive(QObject, Default)]
pub struct DiscordQtApp {
    base: qt_base_class!(trait QObject),

    /// Current user display name
    user_name: qt_property!(QString; NOTIFY user_changed),
    /// Current user avatar URL
    user_avatar: qt_property!(QString; NOTIFY user_changed),
    /// Whether user is logged in
    is_logged_in: qt_property!(bool; NOTIFY user_changed),
    /// Current status (online, idle, dnd, invisible)
    status: qt_property!(QString; NOTIFY status_changed),
    /// Connection state
    connection_state: qt_property!(QString; NOTIFY connection_changed),

    /// User changed signal
    user_changed: qt_signal!(),
    /// Status changed signal
    status_changed: qt_signal!(),
    /// Connection changed signal
    connection_changed: qt_signal!(),
    /// Login requested signal
    login_requested: qt_signal!(),
    /// Logout completed signal
    logout_completed: qt_signal!(),
    /// Error occurred signal
    error_occurred: qt_signal!(message: QString),
    /// Token extracted signal
    token_extracted: qt_signal!(token: QString),

    /// Request login action
    request_login: qt_method!(fn(&self)),
    /// Logout action
    logout: qt_method!(fn(&mut self)),
    /// Set status action
    set_status: qt_method!(fn(&mut self, status: QString)),
    /// Open settings
    open_settings: qt_method!(fn(&self)),
}

impl DiscordQtApp {
    fn request_login(&self) {
        self.login_requested();
    }

    fn logout(&mut self) {
        self.is_logged_in = false;
        self.user_name = QString::default();
        self.user_avatar = QString::default();
        self.user_changed();
        self.logout_completed();
    }

    fn set_status(&mut self, status: QString) {
        self.status = status;
        self.status_changed();
    }

    fn open_settings(&self) {
        // Would open settings dialog
    }

    /// Update user info after login
    pub fn set_user(&mut self, name: &str, avatar: &str) {
        self.user_name = QString::from(name);
        self.user_avatar = QString::from(avatar);
        self.is_logged_in = true;
        self.user_changed();
    }

    /// Update connection state
    pub fn set_connection_state(&mut self, state: &str) {
        self.connection_state = QString::from(state);
        self.connection_changed();
    }

    /// Emit error
    pub fn emit_error(&self, message: &str) {
        self.error_occurred(QString::from(message));
    }

    /// Emit token extracted
    pub fn emit_token_extracted(&self, token: &str) {
        self.token_extracted(QString::from(token));
    }
}

/// Channel list item for Qt model
#[derive(QObject, Default)]
pub struct ChannelItem {
    base: qt_base_class!(trait QObject),

    id: qt_property!(QString),
    name: qt_property!(QString),
    channel_type: qt_property!(i32),
    unread_count: qt_property!(i32),
    is_muted: qt_property!(bool),
}

/// Guild list item for Qt model
#[derive(QObject, Default)]
pub struct GuildItem {
    base: qt_base_class!(trait QObject),

    id: qt_property!(QString),
    name: qt_property!(QString),
    icon_url: qt_property!(QString),
    has_notification: qt_property!(bool),
    mention_count: qt_property!(i32),
}

/// Message item for Qt model
#[derive(QObject, Default)]
pub struct MessageItem {
    base: qt_base_class!(trait QObject),

    id: qt_property!(QString),
    author_name: qt_property!(QString),
    author_avatar: qt_property!(QString),
    content: qt_property!(QString),
    timestamp: qt_property!(QString),
    is_own_message: qt_property!(bool),
    has_attachments: qt_property!(bool),
}

/// User item for friend list
#[derive(QObject, Default)]
pub struct UserItem {
    base: qt_base_class!(trait QObject),

    id: qt_property!(QString),
    username: qt_property!(QString),
    display_name: qt_property!(QString),
    avatar_url: qt_property!(QString),
    status: qt_property!(QString),
    custom_status: qt_property!(QString),
}
