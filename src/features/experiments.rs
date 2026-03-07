// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Experiments — unlock hidden features
//!
//! Discord gates many features behind A/B experiment buckets.
//! This module fetches experiment data and allows overriding
//! bucket assignments to enable hidden features.
//!
//! Detection risk: MEDIUM
//! Fetching experiment endpoints with non-standard flags could be noticed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An experiment assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentAssignment {
    /// Experiment hash/ID
    pub hash: String,
    /// Experiment name (if known)
    pub name: Option<String>,
    /// Current bucket (0 = control, 1+ = treatment groups)
    pub bucket: u32,
    /// Whether this is overridden locally
    pub overridden: bool,
}

/// Experiment override — force a specific bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentOverride {
    pub hash: String,
    pub bucket: u32,
}

/// Manages experiment assignments and overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentManager {
    /// Server-assigned experiments
    pub assignments: HashMap<String, ExperimentAssignment>,
    /// Local overrides (these take priority)
    pub overrides: HashMap<String, u32>,
}

impl ExperimentManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update assignments from server response
    pub fn update_from_response(&mut self, experiments: &serde_json::Value) {
        if let Some(assignments) = experiments.get("assignments").and_then(|v| v.as_array()) {
            for assignment in assignments {
                if let Some(arr) = assignment.as_array() {
                    if arr.len() >= 2 {
                        let hash = arr[0].as_str().unwrap_or("").to_string();
                        let bucket = arr[1].as_u64().unwrap_or(0) as u32;
                        if !hash.is_empty() {
                            let overridden = self.overrides.contains_key(&hash);
                            self.assignments.insert(
                                hash.clone(),
                                ExperimentAssignment {
                                    hash,
                                    name: None,
                                    bucket,
                                    overridden,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    /// Get the effective bucket for an experiment (override takes priority)
    pub fn get_bucket(&self, hash: &str) -> Option<u32> {
        self.overrides
            .get(hash)
            .copied()
            .or_else(|| self.assignments.get(hash).map(|a| a.bucket))
    }

    /// Set a local override for an experiment
    pub fn set_override(&mut self, hash: &str, bucket: u32) {
        self.overrides.insert(hash.to_string(), bucket);
        if let Some(assignment) = self.assignments.get_mut(hash) {
            assignment.overridden = true;
        }
    }

    /// Remove a local override
    pub fn remove_override(&mut self, hash: &str) {
        self.overrides.remove(hash);
        if let Some(assignment) = self.assignments.get_mut(hash) {
            assignment.overridden = false;
        }
    }

    /// Get all overridden experiments
    pub fn get_overrides(&self) -> Vec<(&str, u32)> {
        self.overrides
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect()
    }

    /// Check if an experiment is in treatment (any non-zero bucket)
    pub fn is_in_treatment(&self, hash: &str) -> bool {
        self.get_bucket(hash).map(|b| b > 0).unwrap_or(false)
    }

    /// Number of experiments tracked
    pub fn count(&self) -> usize {
        self.assignments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experiment_override() {
        let mut mgr = ExperimentManager::new();
        mgr.assignments.insert(
            "exp1".to_string(),
            ExperimentAssignment {
                hash: "exp1".to_string(),
                name: Some("Test".to_string()),
                bucket: 0, // control
                overridden: false,
            },
        );

        assert_eq!(mgr.get_bucket("exp1"), Some(0));
        assert!(!mgr.is_in_treatment("exp1"));

        // Override to treatment
        mgr.set_override("exp1", 1);
        assert_eq!(mgr.get_bucket("exp1"), Some(1));
        assert!(mgr.is_in_treatment("exp1"));

        // Remove override
        mgr.remove_override("exp1");
        assert_eq!(mgr.get_bucket("exp1"), Some(0));
    }

    #[test]
    fn test_unknown_experiment() {
        let mgr = ExperimentManager::new();
        assert_eq!(mgr.get_bucket("nonexistent"), None);
        assert!(!mgr.is_in_treatment("nonexistent"));
    }

    #[test]
    fn test_update_from_response() {
        let mut mgr = ExperimentManager::new();
        let response = serde_json::json!({
            "assignments": [
                ["hash_abc", 0, 1, -1, 0],
                ["hash_def", 1, 2, -1, 0]
            ]
        });
        mgr.update_from_response(&response);
        assert_eq!(mgr.count(), 2);
        assert_eq!(mgr.get_bucket("hash_abc"), Some(0));
        assert_eq!(mgr.get_bucket("hash_def"), Some(1));
    }
}
