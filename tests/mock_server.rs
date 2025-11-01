//! Mock server tests (no real API calls needed)

use oanda_connector::{OandaClient, OandaConfig};
use mockito::{Server, Matcher};

async fn create_mock_client(server: &Server) -> OandaClient {
    let mut config = OandaConfig::new(
        "test_api_key".to_string(),
        "test_account_id".to_string(),
        true,
    );
    config.base_url = Some(server.url());
    config.enable_retries = false; // Disable retries for faster tests
    
    OandaClient::new(config).unwrap()
}

#[tokio::test]
async fn test_mock_current_price() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/v3/accounts/test_account_id/pricing")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "prices": [{
                "instrument": "EUR_USD",
                "time": "2024-01-01T12:00:00.000000000Z",
                "bids": [{"price": "1.10000"}],
                "asks": [{"price": "1.10020"}]
            }]
        }"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let tick = client.get_current_price("EUR_USD").await.unwrap();
    
    assert_eq!(tick.instrument, "EUR_USD");
    assert_eq!(tick.bid, 1.10000);
    assert_eq!(tick.ask, 1.10020);
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_authentication_error() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", Matcher::Any)
        .with_status(401)
        .with_body("Unauthorized")
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let result = client.get_current_price("EUR_USD").await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), oanda_connector::Error::AuthenticationFailed));
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_rate_limit() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", Matcher::Any)
        .with_status(429)
        .with_header("Retry-After", "60")
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let result = client.get_current_price("EUR_USD").await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        oanda_connector::Error::RateLimitExceeded { retry_after_seconds } => {
            assert_eq!(retry_after_seconds, 60);
        }
        _ => panic!("Expected RateLimitExceeded error"),
    }
    
    mock.assert_async().await;
}

#[tokio::test]
async fn test_mock_candles() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("GET", "/v3/instruments/EUR_USD/candles")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_body(r#"{
            "instrument": "EUR_USD",
            "granularity": "M5",
            "candles": [{
                "time": "2024-01-01T12:00:00.000000000Z",
                "volume": 100,
                "complete": true,
                "mid": {
                    "o": "1.10000",
                    "h": "1.10050",
                    "l": "1.09950",
                    "c": "1.10020"
                }
            }]
        }"#)
        .create_async()
        .await;
    
    let client = create_mock_client(&server).await;
    let candles = client.get_candles(
        "EUR_USD",
        oanda_connector::Granularity::M5,
        1
    ).await.unwrap();
    
    assert_eq!(candles.len(), 1);
    assert_eq!(candles[0].open, 1.10000);
    assert_eq!(candles[0].close, 1.10020);
    
    mock.assert_async().await;
}