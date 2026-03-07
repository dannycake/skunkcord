// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord X-Super-Properties header generation

use serde::{Deserialize, Serialize};

/// Discord super properties sent with every request
/// These are base64 encoded and sent in the X-Super-Properties header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperProperties {
    pub os: String,
    pub browser: String,
    pub device: String,
    pub system_locale: String,
    pub browser_user_agent: String,
    pub browser_version: String,
    pub os_version: String,
    pub referrer: String,
    pub referring_domain: String,
    pub referrer_current: String,
    pub referring_domain_current: String,
    pub release_channel: String,
    pub client_build_number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_event_source: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub design_id: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Default for SuperProperties {
    fn default() -> Self {
        Self {
            os: "Windows".to_string(),
            browser: "Chrome".to_string(),
            device: "".to_string(),
            system_locale: "en-US".to_string(),
            browser_user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
            browser_version: "131.0.0.0".to_string(),
            os_version: "10".to_string(),
            referrer: "".to_string(),
            referring_domain: "".to_string(),
            referrer_current: "".to_string(),
            referring_domain_current: "".to_string(),
            release_channel: "stable".to_string(),
            client_build_number: 348000,
            client_event_source: None,
            design_id: 0,
        }
    }
}

/// Discord client properties for mobile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSuperProperties {
    pub os: String,
    pub os_version: String,
    pub device: String,
    pub device_vendor_id: String,
    pub browser_user_agent: String,
    pub browser_version: String,
    pub os_sdk_version: String,
    pub client_build_number: u64,
    pub client_version: String,
    pub system_locale: String,
    pub device_advertising_id: String,
}

impl MobileSuperProperties {
    /// Create Android client properties
    pub fn android() -> Self {
        Self {
            os: "Android".to_string(),
            os_version: "14".to_string(),
            device: "Pixel 8 Pro".to_string(),
            device_vendor_id: uuid::Uuid::new_v4().to_string(),
            browser_user_agent: "Discord-Android/220023".to_string(),
            browser_version: "220.23 - rn".to_string(),
            os_sdk_version: "34".to_string(),
            client_build_number: 220023,
            client_version: "220.23 - rn".to_string(),
            system_locale: "en-US".to_string(),
            device_advertising_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create iOS client properties
    pub fn ios() -> Self {
        Self {
            os: "iOS".to_string(),
            os_version: "17.2".to_string(),
            device: "iPhone15,3".to_string(),
            device_vendor_id: uuid::Uuid::new_v4().to_string(),
            browser_user_agent: "Discord-iOS/220023".to_string(),
            browser_version: "220.23".to_string(),
            os_sdk_version: "17.2".to_string(),
            client_build_number: 220023,
            client_version: "220.23".to_string(),
            system_locale: "en-US".to_string(),
            device_advertising_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Discord desktop app properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSuperProperties {
    pub os: String,
    pub browser: String,
    pub release_channel: String,
    pub client_version: String,
    pub os_version: String,
    pub os_arch: String,
    pub app_arch: String,
    pub system_locale: String,
    pub browser_user_agent: String,
    pub browser_version: String,
    pub client_build_number: u64,
    pub native_build_number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_event_source: Option<String>,
    pub design_id: u32,
}

impl Default for DesktopSuperProperties {
    fn default() -> Self {
        Self {
            os: "Windows".to_string(),
            browser: "Discord Client".to_string(),
            release_channel: "stable".to_string(),
            client_version: "1.0.9170".to_string(),
            os_version: "10.0.22631".to_string(),
            os_arch: "x64".to_string(),
            app_arch: "x64".to_string(),
            system_locale: "en-US".to_string(),
            browser_user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) discord/1.0.9170 Chrome/128.0.6613.178 Electron/32.2.7 Safari/537.36".to_string(),
            browser_version: "32.2.7".to_string(),
            client_build_number: 348000,
            native_build_number: 56030,
            client_event_source: None,
            design_id: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    #[test]
    fn test_super_properties_serialization() {
        let props = SuperProperties::default();
        let json = serde_json::to_string(&props).unwrap();
        let encoded = BASE64.encode(json.as_bytes());

        // Verify we can decode it back
        let decoded = BASE64.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        let _: SuperProperties = serde_json::from_str(&decoded_str).unwrap();
    }
}
