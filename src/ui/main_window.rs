// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Main window implementation
//!
//! The primary Discord client window with channels, messages, etc.

use qmetaobject::prelude::*;

/// Main window controller
#[derive(QObject, Default)]
pub struct MainWindow {
    base: qt_base_class!(trait QObject),

    /// Window title
    title: qt_property!(QString; NOTIFY title_changed),
    /// Current guild ID
    current_guild_id: qt_property!(QString; NOTIFY guild_changed),
    /// Current channel ID
    current_channel_id: qt_property!(QString; NOTIFY channel_changed),
    /// Current channel name
    current_channel_name: qt_property!(QString; NOTIFY channel_changed),
    /// Is loading messages
    is_loading_messages: qt_property!(bool; NOTIFY loading_changed),
    /// Message input text
    message_input: qt_property!(QString; NOTIFY input_changed),
    /// Is typing indicator visible
    typing_indicator_visible: qt_property!(bool; NOTIFY typing_changed),
    /// Typing users text
    typing_users_text: qt_property!(QString; NOTIFY typing_changed),

    // Signals
    title_changed: qt_signal!(),
    guild_changed: qt_signal!(),
    channel_changed: qt_signal!(),
    loading_changed: qt_signal!(),
    input_changed: qt_signal!(),
    typing_changed: qt_signal!(),
    new_message_received: qt_signal!(channel_id: QString, author: QString, content: QString),
    notification_received: qt_signal!(title: QString, body: QString, icon: QString),

    // Methods
    select_guild: qt_method!(fn(&mut self, guild_id: QString)),
    select_channel: qt_method!(fn(&mut self, channel_id: QString, channel_type: i32)),
    send_message: qt_method!(fn(&mut self)),
    load_more_messages: qt_method!(fn(&self)),
    start_typing: qt_method!(fn(&self)),
    open_user_profile: qt_method!(fn(&self, user_id: QString)),
    open_guild_settings: qt_method!(fn(&self, guild_id: QString)),
    copy_message_link: qt_method!(fn(&self, message_id: QString)),
    delete_message: qt_method!(fn(&self, message_id: QString)),
    reply_to_message: qt_method!(fn(&mut self, message_id: QString)),
}

impl MainWindow {
    fn select_guild(&mut self, guild_id: QString) {
        self.current_guild_id = guild_id;
        self.guild_changed();
    }

    fn select_channel(&mut self, channel_id: QString, _channel_type: i32) {
        self.current_channel_id = channel_id.clone();
        self.is_loading_messages = true;
        self.channel_changed();
        self.loading_changed();

        // Would trigger message loading
    }

    fn send_message(&mut self) {
        if self.message_input.to_string().trim().is_empty() {
            return;
        }

        // Would send message via client
        self.message_input = QString::default();
        self.input_changed();
    }

    fn load_more_messages(&self) {
        // Would load more messages
    }

    fn start_typing(&self) {
        // Would send typing indicator
    }

    fn open_user_profile(&self, _user_id: QString) {
        // Would open user profile dialog
    }

    fn open_guild_settings(&self, _guild_id: QString) {
        // Would open guild settings
    }

    fn copy_message_link(&self, _message_id: QString) {
        // Would copy message link to clipboard
    }

    fn delete_message(&self, _message_id: QString) {
        // Would delete message
    }

    fn reply_to_message(&mut self, _message_id: QString) {
        // Would set up reply
    }

    /// Update typing indicator
    pub fn update_typing(&mut self, users: Vec<String>) {
        if users.is_empty() {
            self.typing_indicator_visible = false;
        } else {
            self.typing_indicator_visible = true;
            self.typing_users_text = QString::from(
                match users.len() {
                    1 => format!("{} is typing...", users[0]),
                    2 => format!("{} and {} are typing...", users[0], users[1]),
                    3 => format!("{}, {}, and {} are typing...", users[0], users[1], users[2]),
                    _ => "Several people are typing...".to_string(),
                }
                .as_str(),
            );
        }
        self.typing_changed();
    }

    /// Add a new message to the view
    pub fn add_message(&self, channel_id: &str, author: &str, content: &str) {
        self.new_message_received(
            QString::from(channel_id),
            QString::from(author),
            QString::from(content),
        );
    }

    /// Show a notification
    pub fn show_notification(&self, title: &str, body: &str, icon: &str) {
        self.notification_received(
            QString::from(title),
            QString::from(body),
            QString::from(icon),
        );
    }
}

/// Settings dialog controller
#[derive(QObject, Default)]
pub struct SettingsDialog {
    base: qt_base_class!(trait QObject),

    visible: qt_property!(bool; NOTIFY visibility_changed),
    current_tab: qt_property!(QString; NOTIFY tab_changed),

    // App settings
    theme: qt_property!(QString; NOTIFY settings_changed),
    notifications_enabled: qt_property!(bool; NOTIFY settings_changed),
    sounds_enabled: qt_property!(bool; NOTIFY settings_changed),
    close_to_tray: qt_property!(bool; NOTIFY settings_changed),
    hardware_acceleration: qt_property!(bool; NOTIFY settings_changed),

    // Fingerprint settings
    browser_type: qt_property!(QString; NOTIFY fingerprint_changed),
    randomize_fingerprint: qt_property!(bool; NOTIFY fingerprint_changed),
    custom_user_agent: qt_property!(QString; NOTIFY fingerprint_changed),

    // Signals
    visibility_changed: qt_signal!(),
    tab_changed: qt_signal!(),
    settings_changed: qt_signal!(),
    fingerprint_changed: qt_signal!(),
    settings_saved: qt_signal!(),

    // Methods
    show: qt_method!(fn(&mut self)),
    hide: qt_method!(fn(&mut self)),
    select_tab: qt_method!(fn(&mut self, tab: QString)),
    save_settings: qt_method!(fn(&self)),
    reset_to_defaults: qt_method!(fn(&mut self)),
}

impl SettingsDialog {
    fn show(&mut self) {
        self.visible = true;
        self.visibility_changed();
    }

    fn hide(&mut self) {
        self.visible = false;
        self.visibility_changed();
    }

    fn select_tab(&mut self, tab: QString) {
        self.current_tab = tab;
        self.tab_changed();
    }

    fn save_settings(&self) {
        self.settings_saved();
    }

    fn reset_to_defaults(&mut self) {
        self.theme = QString::from("dark");
        self.notifications_enabled = true;
        self.sounds_enabled = true;
        self.close_to_tray = false;
        self.hardware_acceleration = true;
        self.browser_type = QString::from("Chrome");
        self.randomize_fingerprint = false;
        self.custom_user_agent = QString::default();
        self.settings_changed();
        self.fingerprint_changed();
    }
}

/// Account manager dialog
#[derive(QObject, Default)]
pub struct AccountManager {
    base: qt_base_class!(trait QObject),

    visible: qt_property!(bool; NOTIFY visibility_changed),
    selected_account_id: qt_property!(QString; NOTIFY selection_changed),

    visibility_changed: qt_signal!(),
    selection_changed: qt_signal!(),
    account_added: qt_signal!(user_id: QString, username: QString),
    account_removed: qt_signal!(user_id: QString),
    account_switched: qt_signal!(user_id: QString),

    show: qt_method!(fn(&mut self)),
    hide: qt_method!(fn(&mut self)),
    add_account: qt_method!(fn(&self)),
    remove_account: qt_method!(fn(&self, user_id: QString)),
    switch_account: qt_method!(fn(&mut self, user_id: QString)),
}

impl AccountManager {
    fn show(&mut self) {
        self.visible = true;
        self.visibility_changed();
    }

    fn hide(&mut self) {
        self.visible = false;
        self.visibility_changed();
    }

    fn add_account(&self) {
        // Would trigger login flow for new account
    }

    fn remove_account(&self, user_id: QString) {
        self.account_removed(user_id);
    }

    fn switch_account(&mut self, user_id: QString) {
        self.selected_account_id = user_id.clone();
        self.selection_changed();
        self.account_switched(user_id);
    }
}
