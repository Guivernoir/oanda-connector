//! OANDA API endpoint definitions

/// API endpoint paths
pub struct Endpoints;

impl Endpoints {
    /// Get pricing for instruments
    /// GET /v3/accounts/{accountID}/pricing
    pub fn pricing(account_id: &str) -> String {
        format!("/v3/accounts/{}/pricing", account_id)
    }
    
    /// Get candles for an instrument
    /// GET /v3/instruments/{instrument}/candles
    pub fn candles(instrument: &str) -> String {
        format!("/v3/instruments/{}/candles", instrument)
    }
    
    /// Get account summary
    /// GET /v3/accounts/{accountID}
    pub fn account(account_id: &str) -> String {
        format!("/v3/accounts/{}", account_id)
    }
    
    /// Get account instruments
    /// GET /v3/accounts/{accountID}/instruments
    pub fn instruments(account_id: &str) -> String {
        format!("/v3/accounts/{}/instruments", account_id)
    }
    
    /// Create order
    /// POST /v3/accounts/{accountID}/orders
    pub fn orders(account_id: &str) -> String {
        format!("/v3/accounts/{}/orders", account_id)
    }
    
    /// Get open trades
    /// GET /v3/accounts/{accountID}/trades
    pub fn trades(account_id: &str) -> String {
        format!("/v3/accounts/{}/trades", account_id)
    }
    
    /// Get open positions
    /// GET /v3/accounts/{accountID}/positions
    pub fn positions(account_id: &str) -> String {
        format!("/v3/accounts/{}/positions", account_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_formatting() {
        assert_eq!(
            Endpoints::pricing("123-456"),
            "/v3/accounts/123-456/pricing"
        );
        
        assert_eq!(
            Endpoints::candles("EUR_USD"),
            "/v3/instruments/EUR_USD/candles"
        );
    }
}