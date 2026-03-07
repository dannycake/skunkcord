<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Discord Qt — UI and QML

This document describes the QML UI so developers can change layouts, styling, and behavior without breaking the app.

## Entry point and global object

- **Main QML file**: `src/qml/main.qml`. Loaded from `main.rs`; the QML engine has a global property `app` (the `AppController` Rust object).
- **Styling**: Set before Qt starts in `main.rs`: `QT_QUICK_CONTROLS_STYLE=Basic`, `QT_QPA_PLATFORMTHEME=""` so the system theme does not override the dark UI (e.g. no bright yellow focus rectangles).

## Main layout structure

`main.qml` is a single `Window` containing a **RowLayout** with four main areas:

1. **Guild sidebar** (left)  
   - Fixed width `theme.guildBarWidth` (62).  
   - Home/DMs button, separator, guild list (ListView with `guildModel`), user avatar at bottom.  
   - Delegates use the `GuildIcon` component (round icons, pill indicator when active, glow on hover).

2. **Channel sidebar** (middle)  
   - Fixed width `theme.channelBarWidth` (240).  
   - Header with guild name (or "Direct Messages"), then either:
     - **Guild selected**: `channelListView` with `channelModel` (text/voice channels).  
     - **Home selected**: "DIRECT MESSAGES" heading, then either empty state or `dmListView` with `dmChannelModel`.  
   - Bottom: user panel (avatar, username, status, settings gear).

3. **Main content area**  
   - Fills remaining width.  
   - Column: channel header (name, connection status, mark-read), then either:
     - **Voice/Stage channel**: placeholder view (join voice, etc.).  
     - **No channel selected**: centered "Select a channel" empty state.  
     - **Text channel selected**: message list (`messageList` ListView with `messageModel`), then message input bar.

4. **Member list** (right sidebar)  
   - Fixed width ~208 px (min 180, max 280). Visible when `currentGuildId !== "" && currentChannelId !== ""`.  
   - Header "Members — N", then `memberListView` with `memberModel`. Items: `memberId`, `username`, `displayName`, `avatarUrl`.  
   - Populated via **Op 14 (Lazy Guild)** when a guild is selected; backend sends `MembersLoaded`, QML calls `app.consume_members()` in the timer and updates `memberModel` when the consumed data is for the current guild.

So: **channels and DMs are lists in the middle column; messages and input are in the main content column; the member list is in the right column when viewing a server channel.** The message list is visible only when `currentChannelId !== ""` so "Beginning of conversation" and date dividers never appear in the channel list area.

## Theme object

A `QtObject` with `id: theme` defines colors and dimensions used everywhere:

- **Backgrounds**: `bgBase`, `bgPrimary`, `bgSecondary`, `bgTertiary`, `bgHover`, `bgActive`, etc.
- **Text**: `textNormal`, `textSecondary`, `textMuted`, `textFaint`.
- **Accent**: `accent`, `accentHover`, `accentLight`, `accentGlow`, `accentMuted`.
- **Semantic**: `positive`, `warning`, `danger`, `info`.
- **Borders**: `border`, `separator`.
- **Layout**: `guildBarWidth`, `channelBarWidth`, `headerHeight`, `userPanelHeight`, `messageInputH`, `radiusSmall`, `radiusMed`, `radiusLarge`.
- **Animation**: `animFast`, `animNormal`, `animSlow`.

Change these in one place to adjust the whole app. Do not use raw hex or platform-dependent colors for core UI.

## Models (ListModels)

All populated from the backend via `app.consume_*` and `check_for_updates()`:

| Model | Purpose |
|-------|--------|
| `guildModel` | Guild list (left). Items: `guildId`, `name`, `hasUnread`, `mentionCount`. |
| `channelModel` | Channels for selected guild. Items: `channelId`, `name`, `channelType`, `hasUnread`, `mentionCount`. |
| `dmChannelModel` | DM list when Home is selected. Items: `channelId`, `recipientId`, `recipientName`, `recipientAvatarUrl`, etc. |
| `messageModel` | Messages for selected channel. Items: `messageId`, `authorId`, `authorName`, `authorAvatarUrl`, `content`, `timestamp`, `isDeleted`, `messageType`, reply fields, `mentionsMe`, `mentionEveryone`, etc. |
| `memberModel` | Member list (right sidebar) for current guild. Items: `memberId`, `username`, `displayName`, `avatarUrl`. Populated via `consume_members()` from Op 14 Lazy Guild / Op 8 GUILD_MEMBERS_CHUNK. |

QML never fetches from the network; it only calls `app.*` and then reads these models after consuming updates.

## Key QML state (root properties)

These mirror or drive backend state; many are updated by `check_for_updates()` applying `UiUpdate`s:

- `isLoggedIn`, `currentUserId`, `currentUserName`, `currentUserAvatar`, `currentStatus`, `connectionState`
- `currentGuildId`, `currentGuildName`, `currentChannelId`, `currentChannelName`, `currentChannelType`
- `isVoiceChannel` (derived), `replyToMessageId`, `replyToAuthor`, `replyToContent`, `silentMode`
- `messageModel`, `guildModel`, `channelModel`, `dmChannelModel`, `memberModel`

When you add new UI state that the backend must know about, add a corresponding `UiAction` and `AppController` method. When the backend pushes new data, add a `UiUpdate` and handle it in the controller so these properties or models stay in sync.

## Message list behavior

- **ListView** `id: messageList`, `verticalLayoutDirection: ListView.BottomToTop` (newest at bottom).
- **Visibility**: `visible: !isVoiceChannel && currentChannelId !== ""`. Hidden when no channel is selected so the footer and date dividers do not appear in the wrong place.
- **Footer**: Shows "Beginning of conversation" (or "This is the beginning of #channelname") when `!hasMoreHistory && messageModel.count > 0`; shows loading text when `isLoadingMore`.
- **Day dividers**: Use **local** date from timestamps (`localDateStringFromTimestamp` + `formatDateLabel`) so "Today" and date labels match the user’s timezone. Delegates use `showDayDivider` from `messageList.isDayBoundary(index)`.
- **Delegates**: Full message row (avatar, author, timestamp, content, reply bar, system message row, mention highlight) or condensed row; `focus: false` on the delegate and footer to avoid Qt focus rectangles.

## Components (reusable)

Defined in `main.qml` (or the same file as the main window):

- **GuildIcon**: Guild/home icon (round, active state, hover glow). Used in guild sidebar.
- **SettingToggle**: Toggle row for settings popup (label + switch).
- **Twemoji** (if used): Emoji image with fallback text.

Search for `component Foo:` to find all inline components.

## Popups and overlays

- **settingsPopup**: Settings dialog (feature profile, toggles, Log out / Done).
- **quickSwitcherPopup**: Ctrl+K channel/DM switcher.
- **emojiPopup**, **gifPopup**, **editPopup**, **reactionPickerPopup**, **captchaPopup**: Other overlays.

All are `Popup` or similar; they are anchored or centered and do not change the main four-column layout (guild bar, channel bar, main content, member list).

## Where to change what

| Goal | Where to look |
|------|----------------|
| Colors, spacing, radii, animation speed | `theme` QtObject at top of `main.qml` |
| Guild list look (icons, pills, hover) | Guild sidebar ColumnLayout, `GuildIcon` component |
| Channel list or DM list look | `channelListView` delegate, `dmListView` delegate |
| Message row layout (avatar, name, content, reply, system, mention) | `messageList` delegate, `msgDelegate`, `msgRow`, `replyBar`, `systemMsgRow` |
| Day dividers, “Beginning of conversation” | `messageList` footer, day divider in delegate, `localDateStringFromTimestamp` / `formatDateLabel` |
| Input bar (height, focus ring, placeholder) | Rectangle wrapping `messageInput` TextField |
| Headers (guild name, channel name) | Channel sidebar header, main content channel header |
| Empty states (“Select a channel”, “No direct messages”) | Item with `visible: currentChannelId === ""`, DM empty state in channel sidebar |
| Member list (right sidebar) | Fourth column in RowLayout, `memberListView` / `memberModel`, `app.consume_members()` in timer |
| New UI that triggers backend work | Add `UiAction`, handle in bridge, add `AppController` method, call from QML |
| New data shown in UI | Add `UiUpdate`, handle in `AppController`, extend `consume_*` or properties, update QML bindings or ListModels |

## Fonts

- **main.qml**: `FontLoader` for Figtree (or fallback). `fontFamily` is used for most text.  
- **ui_test**: Can use a different font (e.g. JetBrains Mono). Keep `main.qml` font in sync if you want the same look in both.

## Avoiding layout mistakes

- Do not show the message list when `currentChannelId === ""`; use the empty state instead so "Beginning of conversation" and date dividers only appear in the message pane.
- Use **local** date for day boundaries and labels (`localDateStringFromTimestamp` + `formatDateLabel`), not raw UTC date strings.
- Use `theme.separator` (or other theme colors) for dividers; set `focus: false` on list and delegate roots to avoid system focus styling (e.g. yellow lines).
