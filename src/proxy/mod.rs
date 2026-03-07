// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Proxy support module
//!
//! Handles SOCKS5 proxy configuration including Mullvad server selection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Whether proxy is enabled
    pub enabled: bool,
    /// Proxy type
    pub proxy_type: ProxyType,
    /// Proxy host
    pub host: String,
    /// Proxy port
    pub port: u16,
    /// Username (optional)
    pub username: Option<String>,
    /// Password (optional)
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            proxy_type: ProxyType::Socks5,
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        }
    }
}

impl ProxyConfig {
    /// Create a new SOCKS5 proxy config
    pub fn socks5(host: impl Into<String>, port: u16) -> Self {
        Self {
            enabled: true,
            proxy_type: ProxyType::Socks5,
            host: host.into(),
            port,
            username: None,
            password: None,
        }
    }

    /// Create a proxy config with authentication
    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Create from Mullvad server
    pub fn from_mullvad(server: &MullvadServer) -> Self {
        Self {
            enabled: true,
            proxy_type: ProxyType::Socks5,
            host: server.socks_address.clone(),
            port: server.socks_port,
            username: None,
            password: None,
        }
    }

    /// Get the proxy URL
    pub fn to_url(&self) -> String {
        let scheme = match self.proxy_type {
            ProxyType::Socks5 => "socks5",
            ProxyType::Socks5h => "socks5h",
            ProxyType::Http => "http",
            ProxyType::Https => "https",
        };

        if let (Some(ref user), Some(ref pass)) = (&self.username, &self.password) {
            format!("{}://{}:{}@{}:{}", scheme, user, pass, self.host, self.port)
        } else {
            format!("{}://{}:{}", scheme, self.host, self.port)
        }
    }

    /// Create reqwest Proxy object
    pub fn to_reqwest_proxy(&self) -> Result<reqwest::Proxy, reqwest::Error> {
        let proxy = reqwest::Proxy::all(self.to_url())?;
        Ok(proxy)
    }
}

/// Proxy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    /// SOCKS5 proxy (DNS resolved locally)
    Socks5,
    /// SOCKS5 proxy (DNS resolved by proxy)
    Socks5h,
    /// HTTP proxy
    Http,
    /// HTTPS proxy
    Https,
}

/// Mullvad VPN server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MullvadServer {
    /// Server hostname
    pub hostname: String,
    /// Country code (e.g., "us", "de", "gb")
    pub country_code: String,
    /// Country name
    pub country_name: String,
    /// City code (e.g., "nyc", "lon", "fra")
    pub city_code: String,
    /// City name
    pub city_name: String,
    /// SOCKS5 address
    pub socks_address: String,
    /// SOCKS5 port
    pub socks_port: u16,
    /// Whether server is active
    pub active: bool,
    /// Provider (e.g., "31173", "M247")
    pub provider: String,
}

impl MullvadServer {
    /// Get display name
    pub fn display_name(&self) -> String {
        format!(
            "{} - {} ({})",
            self.country_name, self.city_name, self.hostname
        )
    }

    /// Get short display name
    pub fn short_name(&self) -> String {
        format!("{}-{}", self.country_code.to_uppercase(), self.city_code)
    }
}

/// Mullvad server list organized by region
#[derive(Debug, Clone, Default)]
pub struct MullvadServerList {
    /// Servers organized by country code
    pub by_country: HashMap<String, Vec<MullvadServer>>,
    /// All servers
    pub all: Vec<MullvadServer>,
}

impl MullvadServerList {
    /// Create a new server list with built-in Mullvad servers
    pub fn new() -> Self {
        let mut list = Self::default();
        list.load_builtin_servers();
        list
    }

    /// Load the built-in server list
    fn load_builtin_servers(&mut self) {
        // Mullvad SOCKS5 servers - these are the official Mullvad SOCKS5 endpoints
        // Format: socks5://[country-code]-[city-code]-wg.socks5.mullvad.net:1080
        // Note: Requires active Mullvad subscription and being connected to Mullvad VPN

        let servers = vec![
            // United States
            Self::server(
                "us-nyc-wg-001",
                "us",
                "United States",
                "nyc",
                "New York",
                "us-nyc-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-nyc-wg-002",
                "us",
                "United States",
                "nyc",
                "New York",
                "us-nyc-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-lax-wg-001",
                "us",
                "United States",
                "lax",
                "Los Angeles",
                "us-lax-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-lax-wg-002",
                "us",
                "United States",
                "lax",
                "Los Angeles",
                "us-lax-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-chi-wg-001",
                "us",
                "United States",
                "chi",
                "Chicago",
                "us-chi-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-dal-wg-001",
                "us",
                "United States",
                "dal",
                "Dallas",
                "us-dal-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-mia-wg-001",
                "us",
                "United States",
                "mia",
                "Miami",
                "us-mia-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-sea-wg-001",
                "us",
                "United States",
                "sea",
                "Seattle",
                "us-sea-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-sjc-wg-001",
                "us",
                "United States",
                "sjc",
                "San Jose",
                "us-sjc-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-atl-wg-001",
                "us",
                "United States",
                "atl",
                "Atlanta",
                "us-atl-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-den-wg-001",
                "us",
                "United States",
                "den",
                "Denver",
                "us-den-wg.socks5.mullvad.net",
            ),
            Self::server(
                "us-phx-wg-001",
                "us",
                "United States",
                "phx",
                "Phoenix",
                "us-phx-wg.socks5.mullvad.net",
            ),
            // Canada
            Self::server(
                "ca-tor-wg-001",
                "ca",
                "Canada",
                "tor",
                "Toronto",
                "ca-tor-wg.socks5.mullvad.net",
            ),
            Self::server(
                "ca-van-wg-001",
                "ca",
                "Canada",
                "van",
                "Vancouver",
                "ca-van-wg.socks5.mullvad.net",
            ),
            Self::server(
                "ca-mtl-wg-001",
                "ca",
                "Canada",
                "mtl",
                "Montreal",
                "ca-mtl-wg.socks5.mullvad.net",
            ),
            // United Kingdom
            Self::server(
                "gb-lon-wg-001",
                "gb",
                "United Kingdom",
                "lon",
                "London",
                "gb-lon-wg.socks5.mullvad.net",
            ),
            Self::server(
                "gb-lon-wg-002",
                "gb",
                "United Kingdom",
                "lon",
                "London",
                "gb-lon-wg.socks5.mullvad.net",
            ),
            Self::server(
                "gb-man-wg-001",
                "gb",
                "United Kingdom",
                "man",
                "Manchester",
                "gb-man-wg.socks5.mullvad.net",
            ),
            // Germany
            Self::server(
                "de-fra-wg-001",
                "de",
                "Germany",
                "fra",
                "Frankfurt",
                "de-fra-wg.socks5.mullvad.net",
            ),
            Self::server(
                "de-fra-wg-002",
                "de",
                "Germany",
                "fra",
                "Frankfurt",
                "de-fra-wg.socks5.mullvad.net",
            ),
            Self::server(
                "de-ber-wg-001",
                "de",
                "Germany",
                "ber",
                "Berlin",
                "de-ber-wg.socks5.mullvad.net",
            ),
            Self::server(
                "de-dus-wg-001",
                "de",
                "Germany",
                "dus",
                "Dusseldorf",
                "de-dus-wg.socks5.mullvad.net",
            ),
            // Netherlands
            Self::server(
                "nl-ams-wg-001",
                "nl",
                "Netherlands",
                "ams",
                "Amsterdam",
                "nl-ams-wg.socks5.mullvad.net",
            ),
            Self::server(
                "nl-ams-wg-002",
                "nl",
                "Netherlands",
                "ams",
                "Amsterdam",
                "nl-ams-wg.socks5.mullvad.net",
            ),
            // Sweden
            Self::server(
                "se-sto-wg-001",
                "se",
                "Sweden",
                "sto",
                "Stockholm",
                "se-sto-wg.socks5.mullvad.net",
            ),
            Self::server(
                "se-got-wg-001",
                "se",
                "Sweden",
                "got",
                "Gothenburg",
                "se-got-wg.socks5.mullvad.net",
            ),
            Self::server(
                "se-mma-wg-001",
                "se",
                "Sweden",
                "mma",
                "Malmö",
                "se-mma-wg.socks5.mullvad.net",
            ),
            // Switzerland
            Self::server(
                "ch-zrh-wg-001",
                "ch",
                "Switzerland",
                "zrh",
                "Zurich",
                "ch-zrh-wg.socks5.mullvad.net",
            ),
            // France
            Self::server(
                "fr-par-wg-001",
                "fr",
                "France",
                "par",
                "Paris",
                "fr-par-wg.socks5.mullvad.net",
            ),
            Self::server(
                "fr-par-wg-002",
                "fr",
                "France",
                "par",
                "Paris",
                "fr-par-wg.socks5.mullvad.net",
            ),
            // Australia
            Self::server(
                "au-syd-wg-001",
                "au",
                "Australia",
                "syd",
                "Sydney",
                "au-syd-wg.socks5.mullvad.net",
            ),
            Self::server(
                "au-mel-wg-001",
                "au",
                "Australia",
                "mel",
                "Melbourne",
                "au-mel-wg.socks5.mullvad.net",
            ),
            Self::server(
                "au-bne-wg-001",
                "au",
                "Australia",
                "bne",
                "Brisbane",
                "au-bne-wg.socks5.mullvad.net",
            ),
            // Japan
            Self::server(
                "jp-tyo-wg-001",
                "jp",
                "Japan",
                "tyo",
                "Tokyo",
                "jp-tyo-wg.socks5.mullvad.net",
            ),
            Self::server(
                "jp-osa-wg-001",
                "jp",
                "Japan",
                "osa",
                "Osaka",
                "jp-osa-wg.socks5.mullvad.net",
            ),
            // Singapore
            Self::server(
                "sg-sin-wg-001",
                "sg",
                "Singapore",
                "sin",
                "Singapore",
                "sg-sin-wg.socks5.mullvad.net",
            ),
            // Brazil
            Self::server(
                "br-sao-wg-001",
                "br",
                "Brazil",
                "sao",
                "São Paulo",
                "br-sao-wg.socks5.mullvad.net",
            ),
            // Poland
            Self::server(
                "pl-waw-wg-001",
                "pl",
                "Poland",
                "waw",
                "Warsaw",
                "pl-waw-wg.socks5.mullvad.net",
            ),
            // Italy
            Self::server(
                "it-mil-wg-001",
                "it",
                "Italy",
                "mil",
                "Milan",
                "it-mil-wg.socks5.mullvad.net",
            ),
            // Spain
            Self::server(
                "es-mad-wg-001",
                "es",
                "Spain",
                "mad",
                "Madrid",
                "es-mad-wg.socks5.mullvad.net",
            ),
            Self::server(
                "es-bcn-wg-001",
                "es",
                "Spain",
                "bcn",
                "Barcelona",
                "es-bcn-wg.socks5.mullvad.net",
            ),
            // Austria
            Self::server(
                "at-vie-wg-001",
                "at",
                "Austria",
                "vie",
                "Vienna",
                "at-vie-wg.socks5.mullvad.net",
            ),
            // Belgium
            Self::server(
                "be-bru-wg-001",
                "be",
                "Belgium",
                "bru",
                "Brussels",
                "be-bru-wg.socks5.mullvad.net",
            ),
            // Norway
            Self::server(
                "no-osl-wg-001",
                "no",
                "Norway",
                "osl",
                "Oslo",
                "no-osl-wg.socks5.mullvad.net",
            ),
            // Denmark
            Self::server(
                "dk-cph-wg-001",
                "dk",
                "Denmark",
                "cph",
                "Copenhagen",
                "dk-cph-wg.socks5.mullvad.net",
            ),
            // Finland
            Self::server(
                "fi-hel-wg-001",
                "fi",
                "Finland",
                "hel",
                "Helsinki",
                "fi-hel-wg.socks5.mullvad.net",
            ),
            // Ireland
            Self::server(
                "ie-dub-wg-001",
                "ie",
                "Ireland",
                "dub",
                "Dublin",
                "ie-dub-wg.socks5.mullvad.net",
            ),
            // Czech Republic
            Self::server(
                "cz-prg-wg-001",
                "cz",
                "Czech Republic",
                "prg",
                "Prague",
                "cz-prg-wg.socks5.mullvad.net",
            ),
            // Romania
            Self::server(
                "ro-buh-wg-001",
                "ro",
                "Romania",
                "buh",
                "Bucharest",
                "ro-buh-wg.socks5.mullvad.net",
            ),
            // Hong Kong
            Self::server(
                "hk-hkg-wg-001",
                "hk",
                "Hong Kong",
                "hkg",
                "Hong Kong",
                "hk-hkg-wg.socks5.mullvad.net",
            ),
        ];

        for server in servers {
            self.by_country
                .entry(server.country_code.clone())
                .or_default()
                .push(server.clone());
            self.all.push(server);
        }
    }

    /// Create a server entry
    fn server(
        hostname: &str,
        country_code: &str,
        country_name: &str,
        city_code: &str,
        city_name: &str,
        socks_address: &str,
    ) -> MullvadServer {
        MullvadServer {
            hostname: hostname.to_string(),
            country_code: country_code.to_string(),
            country_name: country_name.to_string(),
            city_code: city_code.to_string(),
            city_name: city_name.to_string(),
            socks_address: socks_address.to_string(),
            socks_port: 1080,
            active: true,
            provider: "Mullvad".to_string(),
        }
    }

    /// Get all countries
    pub fn get_countries(&self) -> Vec<(&str, &str)> {
        let mut countries: Vec<(&str, &str)> = self
            .by_country
            .iter()
            .filter_map(|(code, servers)| {
                servers
                    .first()
                    .map(|s| (code.as_str(), s.country_name.as_str()))
            })
            .collect();
        countries.sort_by(|a, b| a.1.cmp(b.1));
        countries
    }

    /// Get cities for a country
    pub fn get_cities(&self, country_code: &str) -> Vec<(&str, &str)> {
        let mut cities: HashMap<&str, &str> = HashMap::new();

        if let Some(servers) = self.by_country.get(country_code) {
            for server in servers {
                cities.insert(&server.city_code, &server.city_name);
            }
        }

        let mut result: Vec<(&str, &str)> = cities.into_iter().collect();
        result.sort_by(|a, b| a.1.cmp(b.1));
        result
    }

    /// Get servers for a country and city
    pub fn get_servers(&self, country_code: &str, city_code: Option<&str>) -> Vec<&MullvadServer> {
        if let Some(servers) = self.by_country.get(country_code) {
            servers
                .iter()
                .filter(|s| city_code.is_none() || Some(s.city_code.as_str()) == city_code)
                .collect()
        } else {
            vec![]
        }
    }

    /// Get a random server for a country
    pub fn get_random_server(&self, country_code: &str) -> Option<&MullvadServer> {
        use rand::seq::SliceRandom;
        self.by_country
            .get(country_code)
            .and_then(|servers| servers.choose(&mut rand::thread_rng()))
    }

    /// Find server by hostname
    pub fn find_server(&self, hostname: &str) -> Option<&MullvadServer> {
        self.all.iter().find(|s| s.hostname == hostname)
    }
}

/// Region groupings for easier selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    NorthAmerica,
    Europe,
    AsiaPacific,
    SouthAmerica,
    All,
}

impl Region {
    /// Get country codes for this region
    pub fn country_codes(&self) -> Vec<&'static str> {
        match self {
            Region::NorthAmerica => vec!["us", "ca"],
            Region::Europe => vec![
                "gb", "de", "nl", "se", "ch", "fr", "pl", "it", "es", "at", "be", "no", "dk", "fi",
                "ie", "cz", "ro",
            ],
            Region::AsiaPacific => vec!["jp", "sg", "au", "hk"],
            Region::SouthAmerica => vec!["br"],
            Region::All => vec![],
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Region::NorthAmerica => "North America",
            Region::Europe => "Europe",
            Region::AsiaPacific => "Asia Pacific",
            Region::SouthAmerica => "South America",
            Region::All => "All Regions",
        }
    }

    /// All regions
    pub fn all() -> Vec<Region> {
        vec![
            Region::All,
            Region::NorthAmerica,
            Region::Europe,
            Region::AsiaPacific,
            Region::SouthAmerica,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_list() {
        let list = MullvadServerList::new();
        assert!(!list.all.is_empty());
        assert!(list.by_country.contains_key("us"));
        assert!(list.by_country.contains_key("de"));
    }

    #[test]
    fn test_proxy_url() {
        let config = ProxyConfig::socks5("127.0.0.1", 1080);
        assert_eq!(config.to_url(), "socks5://127.0.0.1:1080");

        let config_auth = config.with_auth("user", "pass");
        assert_eq!(config_auth.to_url(), "socks5://user:pass@127.0.0.1:1080");
    }
}
