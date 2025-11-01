//! OANDA API client implementation

use crate::{
    config::OandaConfig,
    endpoints::Endpoints,
    error::{Error, Result},
    models::*,
    rate_limiter::RateLimiter,
};
use reqwest::{Client as HttpClient, Response, StatusCode};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// OANDA API client
#[derive(Clone)]
pub struct OandaClient {
    http_client: HttpClient,
    config: Arc<OandaConfig>,
    rate_limiter: Arc<RateLimiter>,
}

impl OandaClient {
    /// Create new OANDA client
    pub fn new(config: OandaConfig) -> Result<Self> {
        config.validate()?;
        
        let http_client = HttpClient::builder()
            .timeout(config.timeout())
            .build()
            .map_err(Error::HttpError)?;
        
        let rate_limiter = Arc::new(RateLimiter::new(config.requests_per_second));
        
        Ok(Self {
            http_client,
            config: Arc::new(config),
            rate_limiter,
        })
    }
    
    /// Get current price for instrument
    /// 
    /// # Arguments
    /// * `instrument` - Instrument name (e.g., "EUR_USD")
    /// 
    /// # Example
    /// ```no_run
    /// use oanda_connector::{OandaClient, OandaConfig};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = OandaConfig::from_env()?;
    ///     let client = OandaClient::new(config)?;
    ///     
    ///     let tick = client.get_current_price("EUR_USD").await?;
    ///     println!("EUR/USD: bid={}, ask={}", tick.bid, tick.ask);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_current_price(&self, instrument: &str) -> Result<Tick> {
        let endpoint = Endpoints::pricing(&self.config.account_id);
        let url = format!("{}{}?instruments={}", self.config.get_base_url(), endpoint, instrument);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Accept-Datetime-Format", "RFC3339")
                .send()
                .await
        }).await?;
        
        let pricing_response: PricingResponse = self.handle_response(response).await?;
        
        pricing_response.prices
            .into_iter()
            .find(|p| p.instrument == instrument)
            .ok_or_else(|| Error::InvalidInstrument(instrument.to_string()))?
            .to_tick()
    }
    
    /// Get multiple current prices
    /// 
    /// # Arguments
    /// * `instruments` - List of instrument names
    pub async fn get_current_prices(&self, instruments: &[String]) -> Result<Vec<Tick>> {
        let endpoint = Endpoints::pricing(&self.config.account_id);
        let instruments_param = instruments.join(",");
        let url = format!("{}{}?instruments={}", 
            self.config.get_base_url(), endpoint, instruments_param);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Accept-Datetime-Format", "RFC3339")
                .send()
                .await
        }).await?;
        
        let pricing_response: PricingResponse = self.handle_response(response).await?;
        
        pricing_response.prices
            .into_iter()
            .map(|p| p.to_tick())
            .collect()
    }
    
    /// Get historical candles for instrument
    /// 
    /// # Arguments
    /// * `instrument` - Instrument name (e.g., "EUR_USD")
    /// * `granularity` - Candle time period
    /// * `count` - Number of candles (max 5000)
    /// 
    /// # Example
    /// ```no_run
    /// use oanda_connector::{OandaClient, OandaConfig, Granularity};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = OandaConfig::from_env()?;
    ///     let client = OandaClient::new(config)?;
    ///     
    ///     let candles = client.get_candles("EUR_USD", Granularity::M5, 100).await?;
    ///     println!("Fetched {} candles", candles.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_candles(
        &self,
        instrument: &str,
        granularity: Granularity,
        count: usize,
    ) -> Result<Vec<Candle>> {
        // OANDA limits to 5000 candles per request
        if count > 5000 {
            return Err(Error::ConfigError(
                format!("Count {} exceeds maximum of 5000", count)
            ));
        }
        
        let endpoint = Endpoints::candles(instrument);
        let url = format!(
            "{}{}?granularity={}&count={}",
            self.config.get_base_url(),
            endpoint,
            granularity,
            count
        );
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Accept-Datetime-Format", "RFC3339")
                .send()
                .await
        }).await?;
        
        let candles_response: CandlesResponse = self.handle_response(response).await?;
        
        candles_response.candles
            .into_iter()
            .map(|c| c.to_candle(instrument.to_string()))
            .collect()
    }
    
    /// Get candles with date range
    /// 
    /// # Arguments
    /// * `instrument` - Instrument name
    /// * `granularity` - Candle time period
    /// * `from` - Start time (RFC3339 format)
    /// * `to` - End time (RFC3339 format)
    pub async fn get_candles_range(
        &self,
        instrument: &str,
        granularity: Granularity,
        from: &str,
        to: &str,
    ) -> Result<Vec<Candle>> {
        let endpoint = Endpoints::candles(instrument);
        let url = format!(
            "{}{}?granularity={}&from={}&to={}",
            self.config.get_base_url(),
            endpoint,
            granularity,
            from,
            to
        );
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Accept-Datetime-Format", "RFC3339")
                .send()
                .await
        }).await?;
        
        let candles_response: CandlesResponse = self.handle_response(response).await?;
        
        candles_response.candles
            .into_iter()
            .map(|c| c.to_candle(instrument.to_string()))
            .collect()
    }
    
    /// Get account summary information
    /// 
    /// # Example
    /// ```no_run
    /// use oanda_connector::{OandaClient, OandaConfig};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = OandaConfig::from_env()?;
    ///     let client = OandaClient::new(config)?;
    ///     
    ///     let summary = client.get_account_summary().await?;
    ///     println!("Balance: {} {}", summary.balance, summary.currency);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_account_summary(&self) -> Result<AccountSummary> {
        let endpoint = Endpoints::account(&self.config.account_id);
        let url = format!("{}{}", self.config.get_base_url(), endpoint);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()
                .await
        }).await?;
        
        let account_response: AccountResponse = self.handle_response(response).await?;
        Ok(account_response.account.to_summary())
    }
    
    /// Get available instruments for the account
    pub async fn get_instruments(&self) -> Result<Vec<Instrument>> {
        let endpoint = Endpoints::instruments(&self.config.account_id);
        let url = format!("{}{}", self.config.get_base_url(), endpoint);
        
        let response = self.request_with_retry(|| async {
            self.rate_limiter.acquire().await;
            
            self.http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()
                .await
        }).await?;
        
        #[derive(serde::Deserialize)]
        struct InstrumentsResponse {
            instruments: Vec<Instrument>,
        }
        
        let instruments_response: InstrumentsResponse = self.handle_response(response).await?;
        Ok(instruments_response.instruments)
    }
    
    /// Check if client is connected and authenticated
    pub async fn health_check(&self) -> Result<bool> {
        match self.get_account_summary().await {
            Ok(_) => Ok(true),
            Err(Error::AuthenticationFailed) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    // ============================================================
    // PRIVATE HELPER METHODS
    // ============================================================
    
    /// Make request with automatic retry logic
    async fn request_with_retry<F, Fut>(&self, mut f: F) -> Result<Response>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = reqwest::Result<Response>>,
    {
        if !self.config.enable_retries {
            return f().await.map_err(Error::HttpError);
        }
        
        let mut attempts = 0;
        let max_attempts = self.config.max_retries + 1;
        
        loop {
            attempts += 1;
            
            match f().await {
                Ok(response) => return Ok(response),
                Err(e) if attempts >= max_attempts => {
                    return Err(Error::HttpError(e));
                }
                Err(e) if e.is_timeout() => {
                    // Exponential backoff for timeouts
                    let delay = Duration::from_millis(100 * 2u64.pow(attempts - 1));
                    sleep(delay).await;
                    continue;
                }
                Err(e) if e.is_connect() => {
                    // Network error, retry with backoff
                    let delay = Duration::from_millis(500 * 2u64.pow(attempts - 1));
                    sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    // Other errors, don't retry
                    return Err(Error::HttpError(e));
                }
            }
        }
    }
    
    /// Handle HTTP response and convert to typed result
    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        
        match status {
            StatusCode::OK => {
                response.json::<T>().await.map_err(|e| Error::ApiError {
                    code: 0,
                    message: format!("Failed to parse response: {}", e),
                })
            }
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await.unwrap_or_default();
                Err(Error::ApiError {
                    code: 400,
                    message: error_text,
                })
            }
            StatusCode::UNAUTHORIZED => {
                Err(Error::AuthenticationFailed)
            }
            StatusCode::FORBIDDEN => {
                Err(Error::AuthenticationFailed)
            }
            StatusCode::NOT_FOUND => {
                let error_text = response.text().await.unwrap_or_default();
                Err(Error::ApiError {
                    code: 404,
                    message: format!("Resource not found: {}", error_text),
                })
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60);
                
                Err(Error::RateLimitExceeded {
                    retry_after_seconds: retry_after,
                })
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                Err(Error::ApiError {
                    code: 500,
                    message: "OANDA server error".to_string(),
                })
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                Err(Error::ApiError {
                    code: 503,
                    message: "OANDA service temporarily unavailable".to_string(),
                })
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(Error::ApiError {
                    code: status.as_u16(),
                    message: error_text,
                })
            }
        }
    }
}

// ============================================================
// BUILDER PATTERN FOR CLIENT
// ============================================================

/// Builder for OandaClient
pub struct OandaClientBuilder {
    config: OandaConfig,
}

impl OandaClientBuilder {
    /// Create new builder with config
    pub fn new(config: OandaConfig) -> Self {
        Self { config }
    }
    
    /// Set timeout
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }
    
    /// Set rate limit
    pub fn rate_limit(mut self, requests_per_second: u32) -> Self {
        self.config.requests_per_second = requests_per_second;
        self
    }
    
    /// Enable/disable retries
    pub fn retries(mut self, enable: bool) -> Self {
        self.config.enable_retries = enable;
        self
    }
    
    /// Set max retry attempts
    pub fn max_retries(mut self, max: u32) -> Self {
        self.config.max_retries = max;
        self
    }
    
    /// Build client
    pub fn build(self) -> Result<OandaClient> {
        OandaClient::new(self.config)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OandaConfig {
        OandaConfig {
            api_key: "test_api_key".to_string(),
            account_id: "test_account_id".to_string(),
            practice: true,
            base_url: None,
            timeout_seconds: 10,
            requests_per_second: 100,
            enable_retries: true,
            max_retries: 3,
        }
    }

    #[test]
    fn test_client_creation() {
        let config = test_config();
        let client = OandaClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_builder() {
        let config = test_config();
        let client = OandaClientBuilder::new(config)
            .timeout(20)
            .rate_limit(50)
            .retries(false)
            .build();
        
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = test_config();
        config.api_key = String::new();
        
        let result = OandaClient::new(config);
        assert!(result.is_err());
    }
}