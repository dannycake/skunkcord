// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Browser Fingerprint Module
//!
//! Generates and manages browser fingerprints for Discord API emulation.
//! This module helps avoid detection by mimicking real browser behavior.

pub mod browser_data;
pub mod super_properties;

pub use browser_data::*;
pub use super_properties::*;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents a complete browser fingerprint used for Discord requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    /// User agent string
    pub user_agent: String,
    /// Browser name (e.g., "Chrome", "Firefox")
    pub browser: String,
    /// Browser version
    pub browser_version: String,
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Device type
    pub device: String,
    /// System locale
    pub system_locale: String,
    /// Browser locale
    pub browser_locale: String,
    /// Client build number
    pub client_build_number: u64,
    /// Release channel (stable, ptb, canary)
    pub release_channel: String,
    /// X-Super-Properties header value (base64 encoded)
    pub x_super_properties: String,
    /// Screen resolution
    pub screen_resolution: (u32, u32),
    /// Color depth
    pub color_depth: u8,
    /// Timezone offset in minutes
    pub timezone_offset: i32,
    /// WebGL vendor
    pub webgl_vendor: String,
    /// WebGL renderer  
    pub webgl_renderer: String,
    /// Hardware concurrency (CPU cores)
    pub hardware_concurrency: u8,
    /// Device memory in GB
    pub device_memory: u8,
    /// Canvas fingerprint hash
    pub canvas_hash: String,
    /// Audio fingerprint hash
    pub audio_hash: String,
}

impl BrowserFingerprint {
    /// Create a new randomized fingerprint based on Chrome browser
    pub fn new_chrome() -> Self {
        let mut rng = rand::thread_rng();

        let chrome_version = Self::random_chrome_version(&mut rng);
        let os = Self::random_os(&mut rng);
        let (os_name, os_version) = os;

        let user_agent = format!(
            "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
            Self::os_to_ua_string(os_name, os_version),
            chrome_version
        );

        let screen_resolutions = [
            (1920, 1080),
            (2560, 1440),
            (1366, 768),
            (1536, 864),
            (1440, 900),
            (1280, 720),
            (3840, 2160),
        ];
        let screen_resolution = screen_resolutions[rng.gen_range(0..screen_resolutions.len())];

        let hardware_concurrency = [4, 8, 12, 16][rng.gen_range(0..4)];
        let device_memory = [4, 8, 16, 32][rng.gen_range(0..4)];

        let canvas_hash = Self::generate_canvas_hash(&mut rng);
        let audio_hash = Self::generate_audio_hash(&mut rng);

        let client_build_number = Self::get_current_build_number();

        let mut fp = Self {
            user_agent: user_agent.clone(),
            browser: "Chrome".to_string(),
            browser_version: chrome_version.to_string(),
            os: os_name.to_string(),
            os_version: os_version.to_string(),
            device: "".to_string(),
            system_locale: "en-US".to_string(),
            browser_locale: "en-US".to_string(),
            client_build_number,
            release_channel: "stable".to_string(),
            x_super_properties: String::new(),
            screen_resolution,
            color_depth: 24,
            timezone_offset: rng.gen_range(-12..=12) * 60,
            webgl_vendor: "Google Inc. (NVIDIA)".to_string(),
            webgl_renderer: Self::random_gpu(&mut rng).to_string(),
            hardware_concurrency,
            device_memory,
            canvas_hash,
            audio_hash,
        };

        fp.x_super_properties = fp.generate_super_properties();
        fp
    }

    /// Create a fingerprint from extracted browser session
    pub fn from_session(
        user_agent: String,
        _cookies: &[(&str, &str)],
        local_storage: &serde_json::Value,
    ) -> Self {
        let mut fp = Self::new_chrome();
        fp.user_agent = user_agent;

        // Extract browser info from user agent
        if let Some(caps) = regex::Regex::new(r"Chrome/(\d+\.\d+\.\d+\.\d+)")
            .ok()
            .and_then(|re| re.captures(&fp.user_agent))
        {
            fp.browser_version = caps[1].to_string();
        }

        // Extract fingerprint from local storage if available
        if let Some(fingerprint) = local_storage.get("fingerprint") {
            if let Some(fp_str) = fingerprint.as_str() {
                fp.canvas_hash = fp_str.to_string();
            }
        }

        fp.x_super_properties = fp.generate_super_properties();
        fp
    }

    /// Generate X-Super-Properties header value
    fn generate_super_properties(&self) -> String {
        let props = SuperProperties {
            os: self.os.clone(),
            browser: self.browser.clone(),
            device: self.device.clone(),
            system_locale: self.system_locale.clone(),
            browser_user_agent: self.user_agent.clone(),
            browser_version: self.browser_version.clone(),
            os_version: self.os_version.clone(),
            referrer: "".to_string(),
            referring_domain: "".to_string(),
            referrer_current: "".to_string(),
            referring_domain_current: "".to_string(),
            release_channel: self.release_channel.clone(),
            client_build_number: self.client_build_number,
            client_event_source: None,
            design_id: 0,
        };

        let json = serde_json::to_string(&props).unwrap_or_default();
        BASE64.encode(json.as_bytes())
    }

    fn random_chrome_version(rng: &mut impl Rng) -> String {
        // Chrome versions as of early 2026: 130-133 are current
        let major = rng.gen_range(130..=133);
        let minor = 0;
        let build = rng.gen_range(6700..6850);
        let patch = rng.gen_range(50..200);
        format!("{}.{}.{}.{}", major, minor, build, patch)
    }

    fn random_os(rng: &mut impl Rng) -> (&'static str, &'static str) {
        // Weighted towards Windows (most common Discord platform)
        let options = [
            ("Windows", "10"),
            ("Windows", "10"),
            ("Windows", "11"),
            ("Windows", "11"),
            ("Windows", "11"),
            ("Mac OS X", "15.0"),
            ("Mac OS X", "14.0"),
            ("Linux", "x86_64"),
        ];
        options[rng.gen_range(0..options.len())]
    }

    fn os_to_ua_string(os: &str, version: &str) -> String {
        match os {
            "Windows" => {
                let nt = if version == "11" { "10.0" } else { "10.0" };
                format!("Windows NT {}; Win64; x64", nt)
            }
            "Mac OS X" => format!("Macintosh; Intel Mac OS X {}", version.replace('.', "_")),
            "Linux" => "X11; Linux x86_64".to_string(),
            _ => "Windows NT 10.0; Win64; x64".to_string(),
        }
    }

    fn random_gpu(rng: &mut impl Rng) -> &'static str {
        let gpus = [
            "ANGLE (NVIDIA GeForce RTX 5090 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (NVIDIA GeForce RTX 4070 Ti SUPER Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (AMD Radeon RX 9070 XT Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (AMD Radeon RX 7900 XTX Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (AMD Radeon RX 7800 XT Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (Intel(R) Arc(TM) A770 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0)",
            "ANGLE (Apple M4 Pro)",
            "ANGLE (Apple M3 Max)",
            "ANGLE (Apple M2 Pro)",
        ];
        gpus[rng.gen_range(0..gpus.len())]
    }

    fn generate_canvas_hash(rng: &mut impl Rng) -> String {
        let data: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        format!("{:x}", hasher.finalize())
    }

    fn generate_audio_hash(rng: &mut impl Rng) -> String {
        let value: f64 = rng.gen_range(0.0..1.0);
        format!("{:.16}", value)
    }

    fn get_current_build_number() -> u64 {
        // Fallback build number — updated by CI/CD every 6 hours.
        // Use BuildNumberFetcher for the live value at runtime.
        348000
    }

    /// Get headers for Discord API requests
    pub fn get_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("User-Agent", self.user_agent.clone()),
            ("X-Super-Properties", self.x_super_properties.clone()),
            ("X-Discord-Locale", self.browser_locale.clone()),
            ("X-Discord-Timezone", "America/New_York".to_string()),
            ("Accept", "*/*".to_string()),
            (
                "Accept-Language",
                format!("{},en;q=0.9", self.browser_locale),
            ),
            ("Accept-Encoding", "gzip, deflate, br".to_string()),
            (
                "Sec-Ch-Ua",
                format!(
                    "\"Chromium\";v=\"{ver}\", \"Google Chrome\";v=\"{ver}\", \"Not-A.Brand\";v=\"99\"",
                    ver = self.browser_version.split('.').next().unwrap_or("131")
                ),
            ),
            ("Sec-Ch-Ua-Mobile", "?0".to_string()),
            ("Sec-Ch-Ua-Platform", format!("\"{}\"", self.os)),
            ("Sec-Fetch-Dest", "empty".to_string()),
            ("Sec-Fetch-Mode", "cors".to_string()),
            ("Sec-Fetch-Site", "same-origin".to_string()),
        ]
    }
}

impl Default for BrowserFingerprint {
    fn default() -> Self {
        Self::new_chrome()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_generation() {
        let fp = BrowserFingerprint::new_chrome();
        assert!(!fp.user_agent.is_empty());
        assert!(!fp.x_super_properties.is_empty());
        assert!(fp.client_build_number > 0);
    }
}
