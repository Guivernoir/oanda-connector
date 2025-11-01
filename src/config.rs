//! Configuration for OANDA connector

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OandaConfig {
    /// OANDA API key (Bearer token)
    pub api_key: String,
    
    /// OANDA account ID
    pub account_id: String,
    
    /// Use practice account (true) or live (false)
    pub practice: bool,
    
    /// Base URL (auto-set based on practice flag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    
    /// Maximum requests per second
    #[serde(default = "default_rate_limit")]
    pub requests_per_second: u32,
    
    /// Enable automatic retries
    #[serde(default = "default_true")]
    pub enable_retries: bool,
    
    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u64 { 10 }
fn default_rate_limit() -> u32 { 100 }
fn default_true() -> bool { true }
fn default_max_retries() -> u32 { 3 }

impl OandaConfig {
    /// Create new configuration
    pub fn new(api_key: String, account_id: String, practice: bool) -> Self {
        Self {
            api_key,
            account_id,
            practice,
            base_url: None,
            timeout_seconds: default_timeout(),
            requests_per_second: default_rate_limit(),
            enable_retries: default_true(),
            max_retries: default_max_retries(),
        }
    }
    
    /// Load configuration from environment variables
    /// 
    /// Expected env vars:
    /// - OANDA_API_KEY (required)
    /// - OANDA_ACCOUNT_ID (required)
    /// - OANDA_PRACTICE (optional, default: true)
    /// - OANDA_TIMEOUT_SECONDS (optional, default: 10)
    /// - OANDA_REQUESTS_PER_SECOND (optional, default: 100)
    pub fn from_env() -> crate::Result<Self> {
        let api_key = std::env::var("OANDA_API_KEY")
            .map_err(|_| crate::Error::ConfigError(
                "OANDA_API_KEY environment variable not set".to_string()
            ))?;
        
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .map_err(|_| crate::Error::ConfigError(
                "OANDA_ACCOUNT_ID environment variable not set".to_string()
            ))?;
        
        let practice = std::env::var("OANDA_PRACTICE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        
        let timeout_seconds = std::env::var("OANDA_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_timeout());
        
        let requests_per_second = std::env::var("OANDA_REQUESTS_PER_SECOND")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_rate_limit());
        
        Ok(Self {
            api_key,
            account_id,
            practice,
            base_url: None,
            timeout_seconds,
            requests_per_second,
            enable_retries: default_true(),
            max_retries: default_max_retries(),
        })
    }
    
    /// Get base URL based on practice flag
    pub fn get_base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| {
            if self.practice {
                "https://api-fxpractice.oanda.com".to_string()
            } else {
                "https://api-fxtrade.oanda.com".to_string()
            }
        })
    }
    
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.api_key.is_empty() {
            return Err(crate::Error::ConfigError(
                "API key cannot be empty".to_string()
            ));
        }
        
        if self.account_id.is_empty() {
            return Err(crate::Error::ConfigError(
                "Account ID cannot be empty".to_string()
            ));
        }
        
        if self.timeout_seconds == 0 {
            return Err(crate::Error::ConfigError(
                "Timeout must be greater than 0".to_string()
            ));
        }
        
        if self.requests_per_second == 0 {
            return Err(crate::Error::ConfigError(
                "Requests per second must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for OandaConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            account_id: String::new(),
            practice: true,
            base_url: None,
            timeout_seconds: default_timeout(),
            requests_per_second: default_rate_limit(),
            enable_retries: default_true(),
            max_retries: default_max_retries(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_base_url() {
        let config_practice = OandaConfig::new(
            "key".to_string(),
            "id".to_string(),
            true
        );
        assert!(config_practice.get_base_url().contains("fxpractice"));
        
        let config_live = OandaConfig::new(
            "key".to_string(),
            "id".to_string(),
            false
        );
        assert!(config_live.get_base_url().contains("fxtrade"));
    }

    #[test]
    fn test_config_validation() {
        let mut config = OandaConfig::default();
        assert!(config.validate().is_err());
        
        config.api_key = "test_key".to_string();
        config.account_id = "test_id".to_string();
        assert!(config.validate().is_ok());
    }
}