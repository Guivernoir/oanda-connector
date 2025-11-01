//! Error types for OANDA connector

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("OANDA API error {code}: {message}")]
    ApiError {
        code: u16,
        message: String,
    },
    
    #[error("Rate limit exceeded, retry after {retry_after_seconds}s")]
    RateLimitExceeded {
        retry_after_seconds: u64,
    },
    
    #[error("Invalid instrument: {0}")]
    InvalidInstrument(String),
    
    #[error("Invalid granularity: {0}")]
    InvalidGranularity(String),
    
    #[error("Authentication failed: invalid API key or account ID")]
    AuthenticationFailed,
    
    #[error("Network timeout after {0}s")]
    Timeout(u64),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Invalid date range: start={start}, end={end}")]
    InvalidDateRange {
        start: String,
        end: String,
    },
    
    #[error("Insufficient account balance: required={required}, available={available}")]
    InsufficientBalance {
        required: f64,
        available: f64,
    },
}

impl Error {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::HttpError(_) | 
            Error::Timeout(_) | 
            Error::RateLimitExceeded { .. }
        )
    }
    
    /// Check if error is related to authentication
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::AuthenticationFailed)
    }
}