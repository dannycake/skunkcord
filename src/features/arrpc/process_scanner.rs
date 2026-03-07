// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Process scanner for automatic game activity detection
//!
//! Periodically scans running processes and matches them against
//! a database of known games/applications to automatically set
//! Rich Presence activity.

use serde::{Deserialize, Serialize};

/// A detectable game/application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectableGame {
    /// Discord application ID (for rich presence)
    pub id: String,
    /// Display name
    pub name: String,
    /// Process names to match (lowercase)
    pub executables: Vec<ExecutableInfo>,
}

/// Executable matching info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableInfo {
    /// Process name (e.g., "minecraft.exe", "factorio")
    pub name: String,
    /// Operating system ("win32", "linux", "darwin")
    pub os: String,
    /// Whether this is the primary executable
    pub is_launcher: bool,
}

/// Result of a process scan
#[derive(Debug, Clone)]
pub struct DetectedProcess {
    /// The game that was detected
    pub game: DetectableGame,
    /// PID of the matched process
    pub pid: u32,
    /// Process name that matched
    pub process_name: String,
}

/// Process scanner that matches running processes against known games
pub struct ProcessScanner {
    /// Database of known games
    games: Vec<DetectableGame>,
    /// Custom games added by the user
    custom_games: Vec<DetectableGame>,
}

impl ProcessScanner {
    /// Create a new scanner with the built-in game database
    pub fn new() -> Self {
        Self {
            games: builtin_games(),
            custom_games: Vec::new(),
        }
    }

    /// Add a custom game to detect
    pub fn add_custom_game(&mut self, game: DetectableGame) {
        self.custom_games.push(game);
    }

    /// Remove a custom game
    pub fn remove_custom_game(&mut self, game_id: &str) {
        self.custom_games.retain(|g| g.id != game_id);
    }

    /// Get all detectable games (built-in + custom)
    pub fn all_games(&self) -> Vec<&DetectableGame> {
        self.games.iter().chain(self.custom_games.iter()).collect()
    }

    /// Match a process name against the game database
    /// Returns the first matching game
    pub fn match_process(&self, process_name: &str) -> Option<&DetectableGame> {
        let name_lower = process_name.to_lowercase();

        // Check custom games first (user preference)
        for game in &self.custom_games {
            for exe in &game.executables {
                if !exe.is_launcher && name_lower == exe.name.to_lowercase() {
                    return Some(game);
                }
            }
        }

        // Then check built-in games
        for game in &self.games {
            for exe in &game.executables {
                if !exe.is_launcher && name_lower == exe.name.to_lowercase() {
                    return Some(game);
                }
            }
        }

        None
    }

    /// Get the total number of detectable games
    pub fn game_count(&self) -> usize {
        self.games.len() + self.custom_games.len()
    }
}

impl Default for ProcessScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in database of popular detectable games
/// This is a small subset — the full list would be much larger
fn builtin_games() -> Vec<DetectableGame> {
    vec![
        DetectableGame {
            id: "356876176465199104".to_string(),
            name: "Minecraft".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "javaw.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "java".to_string(),
                    os: "linux".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "minecraft-launcher".to_string(),
                    os: "linux".to_string(),
                    is_launcher: true,
                },
            ],
        },
        DetectableGame {
            id: "363410645957484544".to_string(),
            name: "Visual Studio Code".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "code.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "code".to_string(),
                    os: "linux".to_string(),
                    is_launcher: false,
                },
            ],
        },
        DetectableGame {
            id: "356875988590469140".to_string(),
            name: "League of Legends".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "league of legends.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "leagueclient.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: true,
                },
            ],
        },
        DetectableGame {
            id: "356869127241523200".to_string(),
            name: "Valorant".to_string(),
            executables: vec![ExecutableInfo {
                name: "valorant-win64-shipping.exe".to_string(),
                os: "win32".to_string(),
                is_launcher: false,
            }],
        },
        DetectableGame {
            id: "356875570916753438".to_string(),
            name: "Counter-Strike 2".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "cs2.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "cs2".to_string(),
                    os: "linux".to_string(),
                    is_launcher: false,
                },
            ],
        },
        DetectableGame {
            id: "356876590342340608".to_string(),
            name: "Fortnite".to_string(),
            executables: vec![ExecutableInfo {
                name: "fortniteClient-win64-shipping.exe".to_string(),
                os: "win32".to_string(),
                is_launcher: false,
            }],
        },
        DetectableGame {
            id: "432980957394370572".to_string(),
            name: "Spotify".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "spotify.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "spotify".to_string(),
                    os: "linux".to_string(),
                    is_launcher: false,
                },
            ],
        },
        DetectableGame {
            id: "367827983903490050".to_string(),
            name: "OBS Studio".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "obs64.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "obs".to_string(),
                    os: "linux".to_string(),
                    is_launcher: false,
                },
            ],
        },
        DetectableGame {
            id: "356877880938070016".to_string(),
            name: "Factorio".to_string(),
            executables: vec![
                ExecutableInfo {
                    name: "factorio.exe".to_string(),
                    os: "win32".to_string(),
                    is_launcher: false,
                },
                ExecutableInfo {
                    name: "factorio".to_string(),
                    os: "linux".to_string(),
                    is_launcher: false,
                },
            ],
        },
        DetectableGame {
            id: "356876354990473217".to_string(),
            name: "Grand Theft Auto V".to_string(),
            executables: vec![ExecutableInfo {
                name: "gta5.exe".to_string(),
                os: "win32".to_string(),
                is_launcher: false,
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_process() {
        let scanner = ProcessScanner::new();
        let result = scanner.match_process("code");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Visual Studio Code");
    }

    #[test]
    fn test_match_process_case_insensitive() {
        let scanner = ProcessScanner::new();
        let result = scanner.match_process("Code");
        assert!(result.is_some());
    }

    #[test]
    fn test_no_match() {
        let scanner = ProcessScanner::new();
        assert!(scanner.match_process("some_random_process").is_none());
    }

    #[test]
    fn test_custom_game() {
        let mut scanner = ProcessScanner::new();
        scanner.add_custom_game(DetectableGame {
            id: "custom_001".to_string(),
            name: "My Custom Game".to_string(),
            executables: vec![ExecutableInfo {
                name: "mygame".to_string(),
                os: "linux".to_string(),
                is_launcher: false,
            }],
        });

        let result = scanner.match_process("mygame");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "My Custom Game");
    }

    #[test]
    fn test_custom_game_priority() {
        let mut scanner = ProcessScanner::new();
        // Add custom game that overrides a built-in process name
        scanner.add_custom_game(DetectableGame {
            id: "custom_code".to_string(),
            name: "My VS Code Fork".to_string(),
            executables: vec![ExecutableInfo {
                name: "code".to_string(),
                os: "linux".to_string(),
                is_launcher: false,
            }],
        });

        let result = scanner.match_process("code");
        assert!(result.is_some());
        // Custom should take priority
        assert_eq!(result.unwrap().name, "My VS Code Fork");
    }

    #[test]
    fn test_launcher_not_matched() {
        let scanner = ProcessScanner::new();
        // minecraft-launcher is marked as is_launcher, shouldn't match
        let result = scanner.match_process("minecraft-launcher");
        assert!(result.is_none());
    }

    #[test]
    fn test_game_count() {
        let mut scanner = ProcessScanner::new();
        let initial = scanner.game_count();
        assert!(initial > 0);

        scanner.add_custom_game(DetectableGame {
            id: "c1".to_string(),
            name: "Test".to_string(),
            executables: vec![],
        });
        assert_eq!(scanner.game_count(), initial + 1);

        scanner.remove_custom_game("c1");
        assert_eq!(scanner.game_count(), initial);
    }
}
