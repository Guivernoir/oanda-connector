//! OANDA API Connector
//! 
//! High-performance Rust client for OANDA's REST and streaming APIs.
//! Handles rate limiting, retries, and error recovery automatically.

pub mod client;
pub mod config;
pub mod endpoints;
pub mod error;
pub mod models;
pub mod rate_limiter;

// Re-export main types
pub use client::OandaClient;
pub use config::OandaConfig;
pub use error::{Error, Result};
pub use models::{Candle, Tick, Granularity, AccountSummary, Instrument};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Ensure main types are accessible
        let _ = std::any::type_name::<OandaClient>();
        let _ = std::any::type_name::<OandaConfig>();
        let _ = std::any::type_name::<Error>();
    }
}