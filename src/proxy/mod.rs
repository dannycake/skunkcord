use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        }
    }
}

impl ProxyConfig {
    pub fn to_url(&self) -> String {
        if let (Some(ref user), Some(ref pass)) = (&self.username, &self.password) {
            if !user.is_empty() && !pass.is_empty() {
                return format!("socks5://{}:{}@{}:{}", user, pass, self.host, self.port);
            }
        }
        format!("socks5://{}:{}", self.host, self.port)
    }

    pub fn to_reqwest_proxy(&self) -> Result<reqwest::Proxy, reqwest::Error> {
        reqwest::Proxy::all(self.to_url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_url() {
        let config = ProxyConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        };
        assert_eq!(config.to_url(), "socks5://127.0.0.1:1080");
    }

    #[test]
    fn test_proxy_url_with_auth() {
        let config = ProxyConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        assert_eq!(config.to_url(), "socks5://user:pass@127.0.0.1:1080");
    }
}
