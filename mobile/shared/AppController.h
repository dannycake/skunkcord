#ifndef APPCONTROLLER_H
#define APPCONTROLLER_H

#include <QObject>
#include <QString>
#include <QMap>

class AppController : public QObject
{
    Q_OBJECT
    // ── Existing properties ──
    Q_PROPERTY(bool is_logged_in READ isLoggedIn NOTIFY state_changed)
    Q_PROPERTY(QString error_message READ errorMessage NOTIFY state_changed)
    Q_PROPERTY(bool is_loading READ isLoading NOTIFY state_changed)
    Q_PROPERTY(QString user_name READ userName NOTIFY state_changed)
    Q_PROPERTY(QString user_id READ userId NOTIFY state_changed)
    Q_PROPERTY(QString user_avatar READ userAvatar NOTIFY state_changed)
    Q_PROPERTY(QString connection_state READ connectionState NOTIFY state_changed)
    Q_PROPERTY(QString typing_display READ typingDisplay NOTIFY state_changed)
    Q_PROPERTY(QString voice_connection_state READ voiceConnectionState NOTIFY state_changed)
    Q_PROPERTY(QString typing_display_json READ typingDisplayJson NOTIFY state_changed)
    // ── New properties (MFA, captcha, presence) ──
    Q_PROPERTY(bool mfa_required READ mfaRequired NOTIFY state_changed)
    Q_PROPERTY(bool captcha_visible READ captchaVisible NOTIFY state_changed)
    Q_PROPERTY(QString captcha_html READ captchaHtml NOTIFY state_changed)
    Q_PROPERTY(quint32 presence_version READ presenceVersion NOTIFY state_changed)
    Q_PROPERTY(QString login_mode READ loginMode NOTIFY state_changed)

public:
    explicit AppController(QObject *parent = nullptr);
    ~AppController() override;

    // ── Property getters ──
    bool isLoggedIn() const { return m_isLoggedIn; }
    QString errorMessage() const { return m_errorMessage; }
    bool isLoading() const { return m_isLoading; }
    QString userName() const { return m_userName; }
    QString userId() const { return m_userId; }
    QString userAvatar() const { return m_userAvatar; }
    QString connectionState() const { return m_connectionState; }
    QString typingDisplay() const { return m_typingDisplay; }
    QString voiceConnectionState() const { return m_voiceConnectionState; }
    QString typingDisplayJson() const { return m_typingDisplayJson; }
    bool mfaRequired() const { return m_mfaRequired; }
    bool captchaVisible() const { return m_captchaVisible; }
    QString captchaHtml() const { return m_captchaHtml; }
    quint32 presenceVersion() const { return m_presenceVersion; }
    QString loginMode() const { return m_loginMode; }

    // ── Existing methods ──
    Q_INVOKABLE void login(const QString &token);
    Q_INVOKABLE void check_for_updates();
    Q_INVOKABLE QString consume_guilds();
    Q_INVOKABLE QString consume_channels();
    Q_INVOKABLE QString consume_messages();
    Q_INVOKABLE QString consume_loaded_messages();
    Q_INVOKABLE QString consume_dm_channels();
    Q_INVOKABLE QString consume_message_edits();
    Q_INVOKABLE QString consume_message_deletions();
    Q_INVOKABLE QString consume_more_messages();
    Q_INVOKABLE QString consume_voice_state();
    Q_INVOKABLE QString consume_voice_participants();
    Q_INVOKABLE QString consume_voice_stats();
    Q_INVOKABLE QString consume_speaking_users();
    Q_INVOKABLE void search_gifs(const QString &query);
    Q_INVOKABLE QString consume_gifs();
    Q_INVOKABLE void load_sticker_packs();
    Q_INVOKABLE QString consume_sticker_packs();
    Q_INVOKABLE void load_guild_emojis(const QString &guildId);
    Q_INVOKABLE QString consume_guild_emojis();
    Q_INVOKABLE QString get_typing_in_channel(const QString &channelId) const;
    Q_INVOKABLE void update_typing_for_channel(const QString &channelId);
    Q_INVOKABLE void select_guild(const QString &guildId);
    Q_INVOKABLE void select_channel(const QString &channelId, int channelType);
    Q_INVOKABLE void send_message(const QString &channelId, const QString &content);
    Q_INVOKABLE void open_dm(const QString &recipientId);
    Q_INVOKABLE void load_more_messages(const QString &channelId, const QString &beforeMessageId);
    Q_INVOKABLE void delete_message(const QString &channelId, const QString &messageId);
    Q_INVOKABLE void edit_message(const QString &channelId, const QString &messageId, const QString &content);
    Q_INVOKABLE void pin_message(const QString &channelId, const QString &messageId);
    Q_INVOKABLE void unpin_message(const QString &channelId, const QString &messageId);
    Q_INVOKABLE void open_pins(const QString &channelId);
    Q_INVOKABLE QString consume_pins();
    Q_INVOKABLE QString consume_members();
    Q_INVOKABLE QString consume_my_profile();
    Q_INVOKABLE QString consume_reaction_updates();
    Q_INVOKABLE QString consume_unread_updates();
    Q_INVOKABLE void copy_message_link(const QString &channelId, const QString &guildId, const QString &messageId);
    Q_INVOKABLE void reply_to_message(const QString &messageId);
    Q_INVOKABLE void add_reaction(const QString &channelId, const QString &messageId, const QString &emoji);
    Q_INVOKABLE void remove_reaction(const QString &channelId, const QString &messageId, const QString &emoji);
    Q_INVOKABLE void send_message_ex(const QString &channelId, const QString &content, bool silent, const QString &replyTo);
    Q_INVOKABLE void send_message_with_options(const QString &channelId, const QString &content, bool silent,
        const QString &replyTo, const QString &stickerIdsJson, const QString &attachmentPathsJson);
    Q_INVOKABLE void start_typing(const QString &channelId);
    Q_INVOKABLE void logout();
    Q_INVOKABLE void mark_all_read();
    Q_INVOKABLE void set_feature_profile(const QString &profile);
    Q_INVOKABLE void set_status(const QString &status);
    Q_INVOKABLE void set_custom_status(const QString &text);
    Q_INVOKABLE void join_voice(const QString &guildId, const QString &channelId);
    Q_INVOKABLE void leave_voice();
    Q_INVOKABLE void toggle_mute();
    Q_INVOKABLE void toggle_deafen();
    Q_INVOKABLE void update_voice_state(const QString &guildId, const QString &channelId, bool selfMute, bool selfDeaf);
    Q_INVOKABLE void load_mullvad_servers();
    Q_INVOKABLE QString consume_mullvad_servers();
    Q_INVOKABLE void set_proxy_settings(bool enabled, const QString &mode, const QString &mullvadCountry,
        const QString &mullvadCity, const QString &mullvadServer, const QString &customHost, int customPort);
    Q_INVOKABLE QString get_proxy_settings() const;

    // ── MFA / Captcha / Credentials ──
    Q_INVOKABLE void login_credentials(const QString &email, const QString &password);
    Q_INVOKABLE void submit_mfa_code(const QString &code);
    Q_INVOKABLE void submit_captcha(const QString &captchaToken);
    Q_INVOKABLE void set_login_mode(const QString &mode);

    // ── Profile / Friends ──
    Q_INVOKABLE void fetch_user_profile(const QString &userId, const QString &guildId);
    Q_INVOKABLE QString consume_user_profile();
    Q_INVOKABLE QString consume_user_profile_raw();
    Q_INVOKABLE void send_friend_request(const QString &username);

    // ── Presence ──
    Q_INVOKABLE QString get_user_status(const QString &userId) const;

    // ── Plugins ──
    Q_INVOKABLE QString consume_plugin_ui();
    Q_INVOKABLE bool is_plugin_enabled(const QString &pluginId) const;
    Q_INVOKABLE QString get_plugin_list() const;
    Q_INVOKABLE QString get_plugin_enabled_states() const;
    Q_INVOKABLE void set_plugin_enabled(const QString &pluginId, bool enabled);
    Q_INVOKABLE void plugin_button_clicked(const QString &pluginId, const QString &buttonId);
    Q_INVOKABLE void plugin_modal_submitted(const QString &pluginId, const QString &modalId, const QString &fieldsJson);
    Q_INVOKABLE bool consume_plugins_refreshed();
    Q_INVOKABLE QString consume_plugin_updates();
    Q_INVOKABLE void install_plugin(const QString &repoUrl);
    Q_INVOKABLE void refresh_plugins();
    Q_INVOKABLE void check_plugin_updates();

    // ── Display settings ──
    Q_INVOKABLE QString get_deleted_message_style() const;
    Q_INVOKABLE void set_deleted_message_style(const QString &style);

    // ── Voice (fake mute/deafen) ──
    Q_INVOKABLE void toggle_fake_mute();
    Q_INVOKABLE void toggle_fake_deafen();

signals:
    void state_changed();

public slots:
    void onUpdateInvoke(int eventType, const QByteArray &json);

private:
    void onUpdate(int eventType, const QByteArray &json);

    // ── Core state ──
    bool m_isLoggedIn = false;
    QString m_errorMessage;
    bool m_isLoading = false;
    QString m_userName;
    QString m_userId;
    QString m_userAvatar;
    QString m_connectionState;
    QString m_typingDisplay;
    QString m_typingDisplayJson = QStringLiteral("[]");
    QString m_voiceConnectionState;

    // ── MFA / Captcha ──
    bool m_mfaRequired = false;
    QString m_mfaTicket;
    bool m_captchaVisible = false;
    QString m_captchaHtml;
    QString m_loginMode = QStringLiteral("token");

    // ── Presence ──
    QMap<QString, QString> m_userPresence;
    quint32 m_presenceVersion = 0;

    // ── Profile ──
    QString m_pendingUserProfile;
    QString m_pendingUserProfileRaw;

    // ── Plugin UI ──
    QString m_pluginUiJson;  // buffered JSON from PLUGIN_UI_UPDATED events
    bool m_pluginsRefreshed = false;
    QString m_pluginUpdatesJson;

    // ── Buffered data (existing) ──
    QString m_guildsJson;
    QString m_channelsJson;
    QString m_dmChannelsJson;
    QString m_loadedMessagesJson;
    QString m_moreMessagesJson;
    QString m_pinsJson;
    QString m_membersJson;
    QString m_myProfileJson;
    QString m_reactionUpdatesJson;
    QString m_unreadUpdatesJson;
    QString m_voiceStateJson;
    QString m_voiceParticipantsJson;
    QString m_voiceStatsJson;
    QString m_speakingUsersJson;
    QString m_gifsJson;
    QString m_stickerPacksJson;
    QString m_guildEmojisJson;
    QString m_mullvadServersJson;

    QByteArray m_pendingMessagesArray;
    QByteArray m_pendingMessageEditsArray;
    QByteArray m_pendingMessageDeletionsArray;
    QString m_currentChannelForTyping;
};

#endif // APPCONTROLLER_H
