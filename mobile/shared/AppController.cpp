#include "AppController.h"
#include "discord_qt.h"
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMetaObject>

static AppController *s_controller = nullptr;

static void discord_update_forward(int event_type, const char *json) {
    if (s_controller && json) {
        QByteArray data(json);
        QMetaObject::invokeMethod(s_controller, "onUpdateInvoke", Qt::QueuedConnection,
            Q_ARG(int, event_type), Q_ARG(QByteArray, data));
    }
}

void AppController::onUpdateInvoke(int eventType, const QByteArray &json) {
    onUpdate(eventType, json);
}

AppController::AppController(QObject *parent)
    : QObject(parent)
{
    s_controller = this;
    discord_set_update_callback(discord_update_forward);
}

AppController::~AppController() {
    if (s_controller == this)
        s_controller = nullptr;
}

static QString extractPayload(const QByteArray &json, const char *key) {
    QJsonDocument doc = QJsonDocument::fromJson(json);
    if (!doc.isObject()) return QString();
    QJsonObject obj = doc.object();
    QString keyStr = QString::fromUtf8(key);
    if (!obj.contains(keyStr)) return QString();
    QJsonValue v = obj.value(keyStr);
    if (v.isArray())
        return QString::fromUtf8(QJsonDocument(v.toArray()).toJson(QJsonDocument::Compact));
    if (v.isObject())
        return QString::fromUtf8(QJsonDocument(v.toObject()).toJson(QJsonDocument::Compact));
    return QString();
}

void AppController::onUpdate(int eventType, const QByteArray &json) {
    QJsonDocument doc = QJsonDocument::fromJson(json);
    QJsonObject root = doc.object();

    switch (eventType) {
    case DISCORD_EVENT_LOGIN_SUCCESS: {
        if (root.contains("LoginSuccess")) {
            QJsonObject o = root["LoginSuccess"].toObject();
            m_userId = o["user_id"].toString();
            m_userName = o["username"].toString();
            m_userAvatar = o["avatar_url"].toString();
            m_isLoggedIn = true;
            m_errorMessage.clear();
            m_isLoading = false;
        }
        break;
    }
    case DISCORD_EVENT_LOGIN_FAILED:
        if (root.contains("LoginFailed"))
            m_errorMessage = root["LoginFailed"].toString();
        m_isLoading = false;
        break;
    case DISCORD_EVENT_GUILDS_LOADED:
        m_guildsJson = extractPayload(json, "GuildsLoaded");
        if (m_guildsJson == "null") m_guildsJson = "[]";
        break;
    case DISCORD_EVENT_CHANNELS_LOADED:
        m_channelsJson = extractPayload(json, "ChannelsLoaded");
        if (m_channelsJson == "null") m_channelsJson = "[]";
        break;
    case DISCORD_EVENT_DM_CHANNELS_LOADED:
        m_dmChannelsJson = extractPayload(json, "DmChannelsLoaded");
        if (m_dmChannelsJson == "null") m_dmChannelsJson = "[]";
        break;
    case DISCORD_EVENT_MESSAGES_LOADED:
        m_loadedMessagesJson = extractPayload(json, "MessagesLoaded");
        if (m_loadedMessagesJson == "null") m_loadedMessagesJson = "[]";
        break;
    case DISCORD_EVENT_NEW_MESSAGE: {
        if (root.contains("NewMessage")) {
            QJsonValue v = root["NewMessage"];
            if (!m_pendingMessagesArray.isEmpty() && m_pendingMessagesArray != "[]") {
                QJsonArray arr = QJsonDocument::fromJson(m_pendingMessagesArray).array();
                arr.append(v);
                m_pendingMessagesArray = QJsonDocument(arr).toJson(QJsonDocument::Compact);
            } else {
                QJsonArray arr;
                arr.append(v);
                m_pendingMessagesArray = QJsonDocument(arr).toJson(QJsonDocument::Compact);
            }
        }
        break;
    }
    case DISCORD_EVENT_MESSAGE_DELETED: {
        if (root.contains("MessageDeleted")) {
            QJsonObject o = root["MessageDeleted"].toObject();
            QJsonObject item;
            item["channel_id"] = o["channel_id"];
            item["message_id"] = o["message_id"];
            QJsonArray arr = QJsonDocument::fromJson(m_pendingMessageDeletionsArray).array();
            arr.append(item);
            m_pendingMessageDeletionsArray = QJsonDocument(arr).toJson(QJsonDocument::Compact);
        }
        break;
    }
    case DISCORD_EVENT_MESSAGE_EDITED: {
        if (root.contains("MessageEdited")) {
            QJsonObject o = root["MessageEdited"].toObject();
            QJsonArray arr = QJsonDocument::fromJson(m_pendingMessageEditsArray).array();
            arr.append(o);
            m_pendingMessageEditsArray = QJsonDocument(arr).toJson(QJsonDocument::Compact);
        }
        break;
    }
    case DISCORD_EVENT_MORE_MESSAGES_LOADED:
        if (root.contains("MoreMessagesLoaded")) {
            QJsonObject o = root["MoreMessagesLoaded"].toObject();
            m_moreMessagesJson = QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_PINS_LOADED:
        if (root.contains("PinsLoaded")) {
            QJsonObject o = root["PinsLoaded"].toObject();
            m_pinsJson = QString::fromUtf8(QJsonDocument(o["messages"].toArray()).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_MEMBERS_LOADED:
        if (root.contains("MembersLoaded")) {
            QJsonObject o = root["MembersLoaded"].toObject();
            m_membersJson = QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_MY_GUILD_PROFILE:
        if (root.contains("MyGuildProfile")) {
            QJsonObject o = root["MyGuildProfile"].toObject();
            m_myProfileJson = QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_CONNECTED:
        m_connectionState = QStringLiteral("connected");
        break;
    case DISCORD_EVENT_DISCONNECTED:
        m_connectionState = QStringLiteral("disconnected");
        break;
    case DISCORD_EVENT_RECONNECTING:
        m_connectionState = QStringLiteral("reconnecting");
        break;
    case DISCORD_EVENT_VOICE_STATE_CHANGED:
        if (root.contains("VoiceStateChanged"))
            m_voiceConnectionState = root["VoiceStateChanged"].toString();
        break;
    case DISCORD_EVENT_VOICE_CONNECTION_PROGRESS:
        if (root.contains("VoiceConnectionProgress"))
            m_voiceConnectionState = root["VoiceConnectionProgress"].toString();
        break;
    case DISCORD_EVENT_VOICE_PARTICIPANTS_CHANGED:
        if (root.contains("VoiceParticipantsChanged")) {
            QJsonObject o = root["VoiceParticipantsChanged"].toObject();
            m_voiceParticipantsJson = QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_VOICE_STATS:
        if (root.contains("VoiceStats")) {
            QJsonObject o = root["VoiceStats"].toObject();
            m_voiceStatsJson = QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_TYPING_STARTED:
        if (root.contains("TypingStarted")) {
            QJsonObject o = root["TypingStarted"].toObject();
            if (o["channel_id"].toString() == m_currentChannelForTyping)
                m_typingDisplay = o["user_name"].toString() + QLatin1String(" is typing...");
        }
        break;
    case DISCORD_EVENT_MESSAGE_REACTIONS_UPDATED:
        if (root.contains("MessageReactionsUpdated")) {
            QJsonObject o = root["MessageReactionsUpdated"].toObject();
            QJsonArray arr = m_reactionUpdatesJson.isEmpty()
                ? QJsonArray() : QJsonDocument::fromJson(m_reactionUpdatesJson.toUtf8()).array();
            arr.append(o);
            m_reactionUpdatesJson = QString::fromUtf8(QJsonDocument(arr).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_UNREAD_UPDATE:
        if (root.contains("UnreadUpdate")) {
            QJsonObject o = root["UnreadUpdate"].toObject();
            QJsonArray arr = m_unreadUpdatesJson.isEmpty()
                ? QJsonArray() : QJsonDocument::fromJson(m_unreadUpdatesJson.toUtf8()).array();
            arr.append(o);
            m_unreadUpdatesJson = QString::fromUtf8(QJsonDocument(arr).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_GIFS_LOADED:
        m_gifsJson = extractPayload(json, "GifsLoaded");
        if (m_gifsJson == "null") m_gifsJson = "[]";
        break;
    case DISCORD_EVENT_STICKER_PACKS_LOADED:
        m_stickerPacksJson = extractPayload(json, "StickerPacksLoaded");
        if (m_stickerPacksJson == "null") m_stickerPacksJson = "[]";
        break;
    case DISCORD_EVENT_GUILD_EMOJIS_LOADED:
        m_guildEmojisJson = extractPayload(json, "GuildEmojisLoaded");
        if (m_guildEmojisJson == "null") m_guildEmojisJson = "[]";
        break;
    case DISCORD_EVENT_MULLVAD_SERVERS_LOADED:
        if (root.contains("MullvadServersLoaded"))
            m_mullvadServersJson = root["MullvadServersLoaded"].toString();
        break;
    case DISCORD_EVENT_ERROR:
        if (root.contains("Error"))
            m_errorMessage = root["Error"].toString();
        break;
    // ── MFA / Captcha ──
    case DISCORD_EVENT_MFA_REQUIRED:
        if (root.contains("MfaRequired")) {
            QJsonObject o = root["MfaRequired"].toObject();
            m_mfaRequired = true;
            m_mfaTicket = o["ticket"].toString();
            m_isLoading = false;
        }
        break;
    case DISCORD_EVENT_CAPTCHA_REQUIRED:
        if (root.contains("CaptchaRequired")) {
            QJsonObject o = root["CaptchaRequired"].toObject();
            m_captchaVisible = true;
            // Build minimal captcha HTML from sitekey for WebView
            QString sitekey = o["sitekey"].toString();
            m_captchaHtml = QStringLiteral("<html><body><div id='captcha' data-sitekey='") + sitekey + QStringLiteral("'></div></body></html>");
            m_isLoading = false;
        }
        break;
    // ── Presence ──
    case DISCORD_EVENT_PRESENCE_UPDATED:
        if (root.contains("PresenceUpdated")) {
            QJsonObject o = root["PresenceUpdated"].toObject();
            QString uid = o["user_id"].toString();
            QString status = o["status"].toString();
            if (!uid.isEmpty())
                m_userPresence[uid] = status;
            m_presenceVersion++;
        }
        break;
    // ── Profile ──
    case DISCORD_EVENT_USER_PROFILE_LOADED:
        if (root.contains("UserProfileLoaded")) {
            QJsonObject o = root["UserProfileLoaded"].toObject();
            m_pendingUserProfile = o["profile_json"].toString();
            m_pendingUserProfileRaw = o["raw_json"].toString();
        }
        break;
    // ── Plugin UI ──
    case DISCORD_EVENT_PLUGIN_UI_UPDATED:
        if (root.contains("PluginUiUpdated")) {
            QJsonObject o = root["PluginUiUpdated"].toObject();
            // Merge into plugin UI JSON — store the whole event for consume_plugin_ui
            QString pid = o["plugin_id"].toString();
            QJsonObject existing = m_pluginUiJson.isEmpty()
                ? QJsonObject()
                : QJsonDocument::fromJson(m_pluginUiJson.toUtf8()).object();
            QJsonObject entry;
            entry[QStringLiteral("buttons")] = o["buttons"];
            entry[QStringLiteral("modals")] = o["modals"];
            existing[pid] = entry;
            m_pluginUiJson = QString::fromUtf8(QJsonDocument(existing).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_PLUGIN_UI_REMOVED:
        if (root.contains("PluginUiRemoved")) {
            QJsonObject o = root["PluginUiRemoved"].toObject();
            QString pid = o["plugin_id"].toString();
            QJsonObject existing = m_pluginUiJson.isEmpty()
                ? QJsonObject()
                : QJsonDocument::fromJson(m_pluginUiJson.toUtf8()).object();
            existing.remove(pid);
            m_pluginUiJson = QString::fromUtf8(QJsonDocument(existing).toJson(QJsonDocument::Compact));
        }
        break;
    case DISCORD_EVENT_PLUGINS_REFRESHED:
        m_pluginsRefreshed = true;
        break;
    case DISCORD_EVENT_PLUGIN_UPDATES_AVAILABLE:
        if (root.contains("PluginUpdatesAvailable"))
            m_pluginUpdatesJson = root["PluginUpdatesAvailable"].toString();
        break;
    default:
        break;
    }
    emit state_changed();
}

// Need a slot for QueuedConnection (Q_INVOKABLE that takes int and QByteArray)
Q_DECLARE_METATYPE(QByteArray)

void AppController::login(const QString &token) {
    m_isLoading = true;
    m_errorMessage.clear();
    emit state_changed();
    discord_login(token.toUtf8().constData());
}

void AppController::check_for_updates() {
    // Updates are pushed via callback; no polling needed, but we emit so QML can refresh bindings
    emit state_changed();
}

QString AppController::consume_guilds() {
    QString r = m_guildsJson;
    m_guildsJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_channels() {
    QString r = m_channelsJson;
    m_channelsJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_messages() {
    QString r = QString::fromUtf8(m_pendingMessagesArray);
    m_pendingMessagesArray = "[]";
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_loaded_messages() {
    QString r = m_loadedMessagesJson;
    m_loadedMessagesJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_dm_channels() {
    QString r = m_dmChannelsJson;
    m_dmChannelsJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_message_edits() {
    QString r = QString::fromUtf8(m_pendingMessageEditsArray);
    m_pendingMessageEditsArray = "[]";
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_message_deletions() {
    QString r = QString::fromUtf8(m_pendingMessageDeletionsArray);
    m_pendingMessageDeletionsArray = "[]";
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_more_messages() {
    QString r = m_moreMessagesJson;
    m_moreMessagesJson.clear();
    return r.isEmpty() ? QStringLiteral("{}") : r;
}

QString AppController::consume_voice_state() {
    QString r = m_voiceStateJson;
    m_voiceStateJson.clear();
    return r;
}

QString AppController::consume_voice_participants() {
    QString r = m_voiceParticipantsJson;
    m_voiceParticipantsJson.clear();
    return r;
}

QString AppController::consume_voice_stats() {
    QString r = m_voiceStatsJson;
    m_voiceStatsJson.clear();
    return r;
}

QString AppController::consume_speaking_users() {
    QString r = m_speakingUsersJson;
    m_speakingUsersJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

void AppController::search_gifs(const QString &query) {
    discord_search_gifs(query.toUtf8().constData());
}

QString AppController::consume_gifs() {
    QString r = m_gifsJson;
    m_gifsJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

void AppController::load_sticker_packs() {
    discord_load_sticker_packs();
}

QString AppController::consume_sticker_packs() {
    QString r = m_stickerPacksJson;
    m_stickerPacksJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

void AppController::load_guild_emojis(const QString &guildId) {
    discord_load_guild_emojis(guildId.toUtf8().constData());
}

QString AppController::consume_guild_emojis() {
    QString r = m_guildEmojisJson;
    m_guildEmojisJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::get_typing_in_channel(const QString &channelId) const {
    (void)channelId;
    return m_typingDisplay;
}

void AppController::update_typing_for_channel(const QString &channelId) {
    m_currentChannelForTyping = channelId;
    m_typingDisplay.clear();
}

void AppController::select_guild(const QString &guildId) {
    discord_select_guild(guildId.toUtf8().constData());
}

void AppController::select_channel(const QString &channelId, int channelType) {
    discord_select_channel(channelId.toUtf8().constData(), static_cast<unsigned char>(channelType));
}

void AppController::send_message(const QString &channelId, const QString &content) {
    discord_send_message(channelId.toUtf8().constData(), content.toUtf8().constData(), 0);
}

void AppController::open_dm(const QString &recipientId) {
    discord_open_dm(recipientId.toUtf8().constData());
}

void AppController::load_more_messages(const QString &channelId, const QString &beforeMessageId) {
    discord_load_more_messages(channelId.toUtf8().constData(), beforeMessageId.toUtf8().constData());
}

void AppController::delete_message(const QString &channelId, const QString &messageId) {
    discord_delete_message(channelId.toUtf8().constData(), messageId.toUtf8().constData());
}

void AppController::edit_message(const QString &channelId, const QString &messageId, const QString &content) {
    discord_edit_message(channelId.toUtf8().constData(), messageId.toUtf8().constData(), content.toUtf8().constData());
}

void AppController::pin_message(const QString &channelId, const QString &messageId) {
    discord_pin_message(channelId.toUtf8().constData(), messageId.toUtf8().constData());
}

void AppController::unpin_message(const QString &channelId, const QString &messageId) {
    discord_unpin_message(channelId.toUtf8().constData(), messageId.toUtf8().constData());
}

void AppController::open_pins(const QString &channelId) {
    discord_open_pins(channelId.toUtf8().constData());
}

QString AppController::consume_pins() {
    QString r = m_pinsJson;
    m_pinsJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_members() {
    QString r = m_membersJson;
    m_membersJson.clear();
    return r.isEmpty() ? QStringLiteral("{}") : r;
}

QString AppController::consume_my_profile() {
    QString r = m_myProfileJson;
    m_myProfileJson.clear();
    return r.isEmpty() ? QStringLiteral("{}") : r;
}

QString AppController::consume_reaction_updates() {
    QString r = m_reactionUpdatesJson;
    m_reactionUpdatesJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

QString AppController::consume_unread_updates() {
    QString r = m_unreadUpdatesJson;
    m_unreadUpdatesJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

void AppController::copy_message_link(const QString &channelId, const QString &guildId, const QString &messageId) {
    (void)channelId;
    (void)guildId;
    (void)messageId;
    // Platform clipboard in C++ would go here
}

void AppController::reply_to_message(const QString &messageId) {
    (void)messageId;
    // Reply state is typically handled in QML; could add FFI if needed
}

void AppController::add_reaction(const QString &channelId, const QString &messageId, const QString &emoji) {
    discord_add_reaction(channelId.toUtf8().constData(), messageId.toUtf8().constData(), emoji.toUtf8().constData());
}

void AppController::remove_reaction(const QString &channelId, const QString &messageId, const QString &emoji) {
    discord_remove_reaction(channelId.toUtf8().constData(), messageId.toUtf8().constData(), emoji.toUtf8().constData());
}

void AppController::send_message_ex(const QString &channelId, const QString &content, bool silent, const QString &replyTo) {
    if (replyTo.isEmpty())
        discord_send_message(channelId.toUtf8().constData(), content.toUtf8().constData(), silent ? 1 : 0);
    else
        discord_send_message(channelId.toUtf8().constData(), content.toUtf8().constData(), silent ? 1 : 0);
}

void AppController::send_message_with_options(const QString &channelId, const QString &content, bool silent,
    const QString &replyTo, const QString &stickerIdsJson, const QString &attachmentPathsJson) {
    (void)stickerIdsJson;
    (void)attachmentPathsJson;
    send_message_ex(channelId, content, silent, replyTo);
}

void AppController::start_typing(const QString &channelId) {
    discord_start_typing(channelId.toUtf8().constData());
}

void AppController::logout() {
    discord_logout();
    m_isLoggedIn = false;
    m_userName.clear();
    m_userId.clear();
    m_userAvatar.clear();
    m_connectionState.clear();
    emit state_changed();
}

void AppController::mark_all_read() {
    discord_mark_all_read();
}

void AppController::set_feature_profile(const QString &profile) {
    discord_set_feature_profile(profile.toUtf8().constData());
}

void AppController::set_status(const QString &status) {
    discord_set_status(status.toUtf8().constData());
}

void AppController::set_custom_status(const QString &text) {
    discord_set_custom_status(text.toUtf8().constData());
}

void AppController::join_voice(const QString &guildId, const QString &channelId) {
    discord_join_voice(guildId.toUtf8().constData(), channelId.toUtf8().constData());
}

void AppController::leave_voice() {
    discord_leave_voice();
}

void AppController::toggle_mute() {
    discord_toggle_mute();
}

void AppController::toggle_deafen() {
    discord_toggle_deafen();
}

void AppController::update_voice_state(const QString &guildId, const QString &channelId, bool selfMute, bool selfDeaf) {
    (void)guildId;
    (void)channelId;
    (void)selfMute;
    (void)selfDeaf;
    // FFI could add a dedicated function if needed
}

void AppController::load_mullvad_servers() {
    discord_get_mullvad_servers();
}

QString AppController::consume_mullvad_servers() {
    QString r = m_mullvadServersJson;
    m_mullvadServersJson.clear();
    return r.isEmpty() ? QStringLiteral("[]") : r;
}

void AppController::set_proxy_settings(bool enabled, const QString &mode, const QString &mullvadCountry,
    const QString &mullvadCity, const QString &mullvadServer, const QString &customHost, int customPort) {
    (void)enabled;
    (void)mode;
    (void)mullvadCountry;
    (void)mullvadCity;
    (void)mullvadServer;
    (void)customHost;
    (void)customPort;
    // SetProxySettings FFI not implemented in first pass; add discord_set_proxy_settings if needed
}

QString AppController::get_proxy_settings() const {
    return QStringLiteral("{}");
}

// ==================== MFA / Captcha / Credentials ====================

void AppController::login_credentials(const QString &email, const QString &password) {
    // Credential login not supported on mobile FFI — use token login.
    // Emit error so user knows to use token.
    (void)email;
    (void)password;
    m_errorMessage = QStringLiteral("Credential login not available on mobile. Please use a token.");
    emit state_changed();
}

void AppController::submit_mfa_code(const QString &code) {
    m_isLoading = true;
    m_mfaRequired = false;
    emit state_changed();
    discord_submit_mfa_code(m_mfaTicket.toUtf8().constData(), code.toUtf8().constData());
}

void AppController::submit_captcha(const QString &captchaToken) {
    m_isLoading = true;
    m_captchaVisible = false;
    emit state_changed();
    discord_captcha_solved(captchaToken.toUtf8().constData());
}

void AppController::set_login_mode(const QString &mode) {
    m_loginMode = mode;
    emit state_changed();
}

// ==================== Profile ====================

void AppController::fetch_user_profile(const QString &userId, const QString &guildId) {
    discord_fetch_user_profile(userId.toUtf8().constData(), guildId.toUtf8().constData());
}

QString AppController::consume_user_profile() {
    QString r = m_pendingUserProfile;
    m_pendingUserProfile.clear();
    return r;
}

QString AppController::consume_user_profile_raw() {
    QString r = m_pendingUserProfileRaw;
    m_pendingUserProfileRaw.clear();
    return r;
}

void AppController::send_friend_request(const QString &username) {
    // TODO: add discord_send_friend_request FFI if needed
    (void)username;
}

// ==================== Presence ====================

QString AppController::get_user_status(const QString &userId) const {
    return m_userPresence.value(userId.trimmed());
}

// ==================== Plugins ====================

QString AppController::consume_plugin_ui() {
    QString r = m_pluginUiJson;
    // Don't clear — desktop keeps it persistent; QML reads it every poll
    return r.isEmpty() ? QStringLiteral("{}") : r;
}

bool AppController::is_plugin_enabled(const QString &pluginId) const {
    return discord_is_plugin_enabled(pluginId.toUtf8().constData()) != 0;
}

QString AppController::get_plugin_list() const {
    char *json = discord_get_plugin_list();
    QString r = QString::fromUtf8(json);
    discord_free_string(json);
    return r;
}

QString AppController::get_plugin_enabled_states() const {
    char *json = discord_get_plugin_enabled_states();
    QString r = QString::fromUtf8(json);
    discord_free_string(json);
    return r;
}

void AppController::set_plugin_enabled(const QString &pluginId, bool enabled) {
    discord_set_plugin_enabled(pluginId.toUtf8().constData(), enabled ? 1 : 0);
}

void AppController::plugin_button_clicked(const QString &pluginId, const QString &buttonId) {
    discord_plugin_button_clicked(pluginId.toUtf8().constData(), buttonId.toUtf8().constData());
}

void AppController::plugin_modal_submitted(const QString &pluginId, const QString &modalId, const QString &fieldsJson) {
    discord_plugin_modal_submitted(pluginId.toUtf8().constData(), modalId.toUtf8().constData(), fieldsJson.toUtf8().constData());
}

bool AppController::consume_plugins_refreshed() {
    bool r = m_pluginsRefreshed;
    m_pluginsRefreshed = false;
    return r;
}

QString AppController::consume_plugin_updates() {
    QString r = m_pluginUpdatesJson;
    m_pluginUpdatesJson.clear();
    return r;
}

void AppController::install_plugin(const QString &repoUrl) {
    // No-op on mobile: only builtin plugins are available
    (void)repoUrl;
}

void AppController::refresh_plugins() {
    // On mobile: builtins are always loaded; emit state_changed so QML re-reads
    emit state_changed();
}

void AppController::check_plugin_updates() {
    // No-op on mobile: builtins don't have git updates
}

// ==================== Display settings ====================

QString AppController::get_deleted_message_style() const {
    char *style = discord_get_deleted_message_style();
    QString r = QString::fromUtf8(style);
    discord_free_string(style);
    return r;
}

void AppController::set_deleted_message_style(const QString &style) {
    discord_set_deleted_message_style(style.toUtf8().constData());
}

// ==================== Voice (fake mute/deafen) ====================

void AppController::toggle_fake_mute() {
    discord_toggle_fake_mute();
}

void AppController::toggle_fake_deafen() {
    discord_toggle_fake_deafen();
}
