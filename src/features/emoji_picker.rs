// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Emoji picker data and search
//!
//! Provides Unicode emoji data organized by category with search support.
//! Custom guild emojis are handled separately via the API.

use serde::{Deserialize, Serialize};

/// Emoji category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmojiCategory {
    Recent,
    SmileysAndEmotion,
    PeopleAndBody,
    AnimalsAndNature,
    FoodAndDrink,
    TravelAndPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
}

impl EmojiCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Recent => "Recent",
            Self::SmileysAndEmotion => "Smileys",
            Self::PeopleAndBody => "People",
            Self::AnimalsAndNature => "Nature",
            Self::FoodAndDrink => "Food",
            Self::TravelAndPlaces => "Travel",
            Self::Activities => "Activities",
            Self::Objects => "Objects",
            Self::Symbols => "Symbols",
            Self::Flags => "Flags",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Recent => "🕐",
            Self::SmileysAndEmotion => "😀",
            Self::PeopleAndBody => "👋",
            Self::AnimalsAndNature => "🐻",
            Self::FoodAndDrink => "🍕",
            Self::TravelAndPlaces => "✈️",
            Self::Activities => "⚽",
            Self::Objects => "💡",
            Self::Symbols => "❤️",
            Self::Flags => "🏁",
        }
    }

    pub fn all() -> &'static [EmojiCategory] {
        &[
            Self::Recent,
            Self::SmileysAndEmotion,
            Self::PeopleAndBody,
            Self::AnimalsAndNature,
            Self::FoodAndDrink,
            Self::TravelAndPlaces,
            Self::Activities,
            Self::Objects,
            Self::Symbols,
            Self::Flags,
        ]
    }
}

/// A single emoji entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiEntry {
    /// The emoji character(s)
    pub emoji: String,
    /// Short name (e.g., "smile", "thumbsup")
    pub name: String,
    /// Search keywords
    pub keywords: Vec<String>,
    /// Category
    pub category: EmojiCategory,
}

/// Built-in commonly used emojis (subset — full list would be much larger)
pub fn common_emojis() -> Vec<EmojiEntry> {
    vec![
        // Smileys
        emoji(
            "😀",
            "grinning",
            EmojiCategory::SmileysAndEmotion,
            &["happy", "smile"],
        ),
        emoji(
            "😂",
            "joy",
            EmojiCategory::SmileysAndEmotion,
            &["laugh", "cry", "funny"],
        ),
        emoji(
            "🥲",
            "smiling_face_with_tear",
            EmojiCategory::SmileysAndEmotion,
            &["sad", "happy"],
        ),
        emoji(
            "😊",
            "blush",
            EmojiCategory::SmileysAndEmotion,
            &["happy", "shy"],
        ),
        emoji(
            "😭",
            "sob",
            EmojiCategory::SmileysAndEmotion,
            &["cry", "sad"],
        ),
        emoji(
            "🥺",
            "pleading_face",
            EmojiCategory::SmileysAndEmotion,
            &["please", "puppy"],
        ),
        emoji(
            "😤",
            "triumph",
            EmojiCategory::SmileysAndEmotion,
            &["angry", "frustrated"],
        ),
        emoji(
            "💀",
            "skull",
            EmojiCategory::SmileysAndEmotion,
            &["dead", "death"],
        ),
        emoji("🤡", "clown", EmojiCategory::SmileysAndEmotion, &["fool"]),
        emoji(
            "😈",
            "smiling_imp",
            EmojiCategory::SmileysAndEmotion,
            &["devil", "evil"],
        ),
        // Gestures
        emoji(
            "👍",
            "thumbsup",
            EmojiCategory::PeopleAndBody,
            &["yes", "good", "ok"],
        ),
        emoji(
            "👎",
            "thumbsdown",
            EmojiCategory::PeopleAndBody,
            &["no", "bad"],
        ),
        emoji(
            "👋",
            "wave",
            EmojiCategory::PeopleAndBody,
            &["hello", "hi", "bye"],
        ),
        emoji(
            "🙏",
            "pray",
            EmojiCategory::PeopleAndBody,
            &["please", "thanks"],
        ),
        emoji(
            "💪",
            "muscle",
            EmojiCategory::PeopleAndBody,
            &["strong", "flex"],
        ),
        // Hearts
        emoji("❤️", "heart", EmojiCategory::Symbols, &["love", "red"]),
        emoji(
            "🔥",
            "fire",
            EmojiCategory::TravelAndPlaces,
            &["hot", "lit"],
        ),
        emoji(
            "✨",
            "sparkles",
            EmojiCategory::TravelAndPlaces,
            &["magic", "clean"],
        ),
        emoji("💯", "100", EmojiCategory::Symbols, &["perfect", "score"]),
        emoji("⭐", "star", EmojiCategory::TravelAndPlaces, &["favorite"]),
        // Reactions
        emoji(
            "✅",
            "white_check_mark",
            EmojiCategory::Symbols,
            &["yes", "done"],
        ),
        emoji(
            "❌",
            "x",
            EmojiCategory::Symbols,
            &["no", "wrong", "cancel"],
        ),
        emoji(
            "⚠️",
            "warning",
            EmojiCategory::Symbols,
            &["caution", "alert"],
        ),
        emoji(
            "ℹ️",
            "information_source",
            EmojiCategory::Symbols,
            &["info"],
        ),
        emoji(
            "🎉",
            "tada",
            EmojiCategory::Activities,
            &["party", "celebrate"],
        ),
        emoji(
            "🎮",
            "video_game",
            EmojiCategory::Activities,
            &["game", "gaming"],
        ),
        emoji(
            "🎵",
            "musical_note",
            EmojiCategory::Objects,
            &["music", "song"],
        ),
        emoji(
            "📌",
            "pushpin",
            EmojiCategory::Objects,
            &["pin", "important"],
        ),
        emoji("🔗", "link", EmojiCategory::Objects, &["url", "chain"]),
        emoji(
            "💻",
            "computer",
            EmojiCategory::Objects,
            &["laptop", "code"],
        ),
    ]
}

fn emoji(emoji: &str, name: &str, cat: EmojiCategory, kw: &[&str]) -> EmojiEntry {
    EmojiEntry {
        emoji: emoji.to_string(),
        name: name.to_string(),
        keywords: kw.iter().map(|s| s.to_string()).collect(),
        category: cat,
    }
}

/// Search emojis by name or keyword
pub fn search_emojis<'a>(query: &str, emojis: &'a [EmojiEntry]) -> Vec<&'a EmojiEntry> {
    let q = query.to_lowercase();
    emojis
        .iter()
        .filter(|e| {
            e.name.contains(&q) || e.keywords.iter().any(|k| k.contains(&q)) || e.emoji == query
        })
        .collect()
}

/// Recent emoji tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentEmojis {
    /// Ordered list of recently used emoji (most recent first)
    pub emojis: Vec<String>,
    /// Maximum number of recent emojis to track
    pub max_size: usize,
}

impl RecentEmojis {
    pub fn new(max_size: usize) -> Self {
        Self {
            emojis: Vec::new(),
            max_size,
        }
    }

    pub fn use_emoji(&mut self, emoji: &str) {
        self.emojis.retain(|e| e != emoji);
        self.emojis.insert(0, emoji.to_string());
        self.emojis.truncate(self.max_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_by_name() {
        let emojis = common_emojis();
        let results = search_emojis("thumb", &emojis);
        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.name == "thumbsup"));
    }

    #[test]
    fn test_search_by_keyword() {
        let emojis = common_emojis();
        let results = search_emojis("happy", &emojis);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_no_results() {
        let emojis = common_emojis();
        let results = search_emojis("zzzznonexistent", &emojis);
        assert!(results.is_empty());
    }

    #[test]
    fn test_recent_emojis() {
        let mut recent = RecentEmojis::new(5);
        recent.use_emoji("👍");
        recent.use_emoji("❤️");
        recent.use_emoji("😂");
        assert_eq!(recent.emojis[0], "😂"); // Most recent first
        assert_eq!(recent.emojis.len(), 3);

        // Using again moves to front
        recent.use_emoji("👍");
        assert_eq!(recent.emojis[0], "👍");
        assert_eq!(recent.emojis.len(), 3); // No duplicates
    }

    #[test]
    fn test_recent_max_size() {
        let mut recent = RecentEmojis::new(3);
        recent.use_emoji("1");
        recent.use_emoji("2");
        recent.use_emoji("3");
        recent.use_emoji("4");
        assert_eq!(recent.emojis.len(), 3);
        assert_eq!(recent.emojis[0], "4"); // Most recent
        assert!(!recent.emojis.contains(&"1".to_string())); // Oldest dropped
    }

    #[test]
    fn test_categories() {
        let cats = EmojiCategory::all();
        assert!(cats.len() >= 10);
        assert_eq!(cats[0], EmojiCategory::Recent);
    }

    #[test]
    fn test_common_emojis_not_empty() {
        let emojis = common_emojis();
        assert!(emojis.len() >= 20);
    }
}
