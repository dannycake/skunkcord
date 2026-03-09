/**
 * Skunkcord — C FFI header for mobile (iOS/Android)
 *
 * Link against libskunkcord.a (iOS) or libskunkcord.so (Android).
 * Call skunkcord_init() once at app startup, then skunkcord_set_update_callback()
 * to receive UI updates. Use skunkcord_login(token) to start a session.
 * All string arguments are UTF-8. Strings returned by the library must be
 * freed with skunkcord_free_string().
 */

#ifndef SKUNKCORD_H
#define SKUNKCORD_H

#ifdef __cplusplus
extern "C" {
#endif

/* --- Event types for the UI update callback (event_type, json) --- */
#define SKUNKCORD_EVENT_LOGIN_SUCCESS            1
#define SKUNKCORD_EVENT_LOGIN_FAILED             2
#define SKUNKCORD_EVENT_GUILDS_LOADED            3
#define SKUNKCORD_EVENT_CHANNELS_LOADED          4
#define SKUNKCORD_EVENT_DM_CHANNELS_LOADED       5
#define SKUNKCORD_EVENT_MESSAGES_LOADED          6
#define SKUNKCORD_EVENT_NEW_MESSAGE              7
#define SKUNKCORD_EVENT_MESSAGE_DELETED          8
#define SKUNKCORD_EVENT_MESSAGE_EDITED           9
#define SKUNKCORD_EVENT_VOICE_STATE_CHANGED     10
#define SKUNKCORD_EVENT_USER_SPEAKING           11
#define SKUNKCORD_EVENT_UNREAD_UPDATE            12
#define SKUNKCORD_EVENT_CAPTCHA_REQUIRED        13
#define SKUNKCORD_EVENT_CONNECTED                14
#define SKUNKCORD_EVENT_DISCONNECTED             15
#define SKUNKCORD_EVENT_RECONNECTING            16
#define SKUNKCORD_EVENT_MORE_MESSAGES_LOADED    17
#define SKUNKCORD_EVENT_PINS_LOADED              18
#define SKUNKCORD_EVENT_TYPING_STARTED          19
#define SKUNKCORD_EVENT_MESSAGE_REACTIONS_UPDATED 20
#define SKUNKCORD_EVENT_GIFS_LOADED              21
#define SKUNKCORD_EVENT_STICKER_PACKS_LOADED    22
#define SKUNKCORD_EVENT_GUILD_EMOJIS_LOADED     23
#define SKUNKCORD_EVENT_MEMBERS_LOADED           24
#define SKUNKCORD_EVENT_VOICE_PARTICIPANTS_CHANGED 25
#define SKUNKCORD_EVENT_VOICE_CONNECTION_PROGRESS 26
#define SKUNKCORD_EVENT_VOICE_SPEAKING_CHANGED  27
#define SKUNKCORD_EVENT_VOICE_STATS              28
#define SKUNKCORD_EVENT_ERROR                    29
#define SKUNKCORD_EVENT_MY_GUILD_PROFILE        30
#define SKUNKCORD_EVENT_MULLVAD_SERVERS_LOADED  31
#define SKUNKCORD_EVENT_USER_PROFILE_LOADED     32
#define SKUNKCORD_EVENT_PRESENCE_UPDATED        33
#define SKUNKCORD_EVENT_MFA_REQUIRED            34
#define SKUNKCORD_EVENT_PLUGIN_UI_UPDATED       35
#define SKUNKCORD_EVENT_PLUGIN_UI_REMOVED       36
#define SKUNKCORD_EVENT_PLUGINS_REFRESHED       37
#define SKUNKCORD_EVENT_PLUGIN_UPDATES_AVAILABLE 38
#define SKUNKCORD_EVENT_JOIN_GUILD_SUCCESS       39
#define SKUNKCORD_EVENT_JOIN_GUILD_FAILED        40
#define SKUNKCORD_EVENT_RPC_INVITE_RECEIVED      41
#define SKUNKCORD_EVENT_RELATIONSHIPS_UPDATE     42
#define SKUNKCORD_EVENT_GUILD_MUTE_STATE_CHANGED 43
#define SKUNKCORD_EVENT_ACCOUNTS_LIST             44

/** Callback for UI updates. json is valid only for the duration of the call. */
typedef void (*skunkcord_update_callback_t)(int event_type, const char *json);

/* --- Initialization --- */
/** Initialize the library. Call once at startup. Returns 0 on success, -1 on error. */
int skunkcord_init(void);

/** Register callback for UI updates. Call after skunkcord_init(). */
void skunkcord_set_update_callback(skunkcord_update_callback_t callback);

/** Free a string returned by the library. */
void skunkcord_free_string(char *ptr);

/** Get library version (caller must free with skunkcord_free_string). */
char *skunkcord_version(void);

/* --- Login / session --- */
/** Send login token. Returns 0 on success, -1 on failure. */
int skunkcord_login(const char *token);

/** Logout. Returns 0 on success. */
int skunkcord_logout(void);

/* --- Navigation --- */
/** Select guild (empty string = DMs). */
int skunkcord_select_guild(const char *guild_id);

/** Select channel. channel_type: 0 = guild text, 1 = DM, etc. */
int skunkcord_select_channel(const char *channel_id, unsigned char channel_type);

/* --- Messaging --- */
/** Send message. silent: 1 = true, 0 = false. */
int skunkcord_send_message(const char *channel_id, const char *content, int silent);

/** Start typing in channel. */
int skunkcord_start_typing(const char *channel_id);

/** Edit message. */
int skunkcord_edit_message(const char *channel_id, const char *message_id, const char *content);

/** Delete message. */
int skunkcord_delete_message(const char *channel_id, const char *message_id);

/** Pin message. */
int skunkcord_pin_message(const char *channel_id, const char *message_id);

/** Unpin message. */
int skunkcord_unpin_message(const char *channel_id, const char *message_id);

/** Open pinned messages for channel. */
int skunkcord_open_pins(const char *channel_id);

/** Add reaction. emoji: unicode or ":name:id". */
int skunkcord_add_reaction(const char *channel_id, const char *message_id, const char *emoji);

/** Remove own reaction. */
int skunkcord_remove_reaction(const char *channel_id, const char *message_id, const char *emoji);

/* --- Voice --- */
/** Join voice channel. guild_id can be null/empty for group DMs. */
int skunkcord_join_voice(const char *guild_id, const char *channel_id);

/** Leave voice. */
int skunkcord_leave_voice(void);

/** Toggle mute. */
int skunkcord_toggle_mute(void);

/** Toggle deafen. */
int skunkcord_toggle_deafen(void);

/** Toggle fake mute (appear muted but still receive). */
int skunkcord_toggle_fake_mute(void);

/** Toggle fake deafen (appear deafened but still hear). */
int skunkcord_toggle_fake_deafen(void);

/* --- Profile / status --- */
/** Deprecated: no-op. Feature profiles removed. */
int skunkcord_set_feature_profile(const char *profile);

/** Set status (online, idle, dnd, invisible). */
int skunkcord_set_status(const char *status);

/** Set custom status (empty to clear). */
int skunkcord_set_custom_status(const char *text);

/* --- Other --- */
/** Switch account. */
int skunkcord_switch_account(const char *account_id);

/** Mark all as read. */
int skunkcord_mark_all_read(void);

/** Submit captcha solution token. */
int skunkcord_captcha_solved(const char *token);

/** Submit MFA TOTP code. ticket comes from MFA_REQUIRED event JSON. */
int skunkcord_submit_mfa_code(const char *ticket, const char *code);

/** Fetch user profile (for profile popup). guild_id can be empty. */
int skunkcord_fetch_user_profile(const char *user_id, const char *guild_id);

/* --- Plugins --- */
/** Set plugin enabled/disabled. enabled: 1 = true, 0 = false. */
int skunkcord_set_plugin_enabled(const char *plugin_id, int enabled);

/** Notify backend that a plugin toolbar button was clicked. */
int skunkcord_plugin_button_clicked(const char *plugin_id, const char *button_id);

/** Submit a plugin modal form (fields_json: JSON object string). */
int skunkcord_plugin_modal_submitted(const char *plugin_id, const char *modal_id,
                                   const char *fields_json);

/** Set deleted message display style ("strikethrough", "faded", "deleted"). */
int skunkcord_set_deleted_message_style(const char *style);

/* --- Synchronous getters (caller must free returned strings with skunkcord_free_string) --- */

/** Get deleted message display style (caller frees). */
char *skunkcord_get_deleted_message_style(void);

/** Check if a plugin is enabled. Returns 1 (true) or 0 (false). */
int skunkcord_is_plugin_enabled(const char *plugin_id);

/** Get plugin list as JSON array: [{id, name, description}, ...] (caller frees). */
char *skunkcord_get_plugin_list(void);

/** Get plugin enabled states as JSON object: {pluginId: bool} (caller frees). */
char *skunkcord_get_plugin_enabled_states(void);

/** Open DM with user. */
int skunkcord_open_dm(const char *recipient_id);

/** Load more (older) messages. */
int skunkcord_load_more_messages(const char *channel_id, const char *before_message_id);

/** Search GIFs (empty = trending). */
int skunkcord_search_gifs(const char *query);

/** Load sticker packs. */
int skunkcord_load_sticker_packs(void);

/** Load guild emojis. */
int skunkcord_load_guild_emojis(const char *guild_id);

/** Request Mullvad server list. */
int skunkcord_get_mullvad_servers(void);

/* --- Platform (super properties) --- */
/** Android super properties JSON (caller frees). */
char *skunkcord_android_super_properties(void);

/** iOS super properties JSON (caller frees). */
char *skunkcord_ios_super_properties(void);

#ifdef __cplusplus
}
#endif

#endif /* SKUNKCORD_H */
