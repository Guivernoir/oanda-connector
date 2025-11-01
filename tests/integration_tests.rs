//! Integration tests for OANDA connector
//! 
//! These tests require valid OANDA credentials in environment:
//! - OANDA_API_KEY
//! - OANDA_ACCOUNT_ID
//! - OANDA_PRACTICE=true (recommended)

use oanda_connector::{OandaClient, OandaConfig, Granularity};
use std::time::Duration;

fn get_test_client() -> OandaClient {
    let config = OandaConfig::from_env()
        .expect("OANDA credentials not set. Set OANDA_API_KEY and OANDA_ACCOUNT_ID");
    
    OandaClient::new(config).expect("Failed to create client")
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --nocapture
async fn test_health_check() {
    let client = get_test_client();
    
    let result = client.health_check().await;
    assert!(result.is_ok(), "Health check failed: {:?}", result);
    assert!(result.unwrap(), "Health check returned false");
}

#[tokio::test]
#[ignore]
async fn test_get_account_summary() {
    let client = get_test_client();
    
    let summary = client.get_account_summary().await
        .expect("Failed to get account summary");
    
    assert!(!summary.id.is_empty());
    assert!(!summary.currency.is_empty());
    println!("Account balance: {} {}", summary.balance, summary.currency);
}

#[tokio::test]
#[ignore]
async fn test_get_current_price() {
    let client = get_test_client();
    
    let tick = client.get_current_price("EUR_USD").await
        .expect("Failed to get current price");
    
    assert_eq!(tick.instrument, "EUR_USD");
    assert!(tick.bid > 0.0);
    assert!(tick.ask > 0.0);
    assert!(tick.ask > tick.bid, "Ask should be greater than bid");
    assert!(tick.spread() > 0.0);
    
    println!("EUR/USD: bid={}, ask={}, spread={}", tick.bid, tick.ask, tick.spread());
}

#[tokio::test]
#[ignore]
async fn test_get_multiple_prices() {
    let client = get_test_client();
    
    let instruments = vec![
        "EUR_USD".to_string(),
        "GBP_USD".to_string(),
        "USD_JPY".to_string(),
    ];
    
    let ticks = client.get_current_prices(&instruments).await
        .expect("Failed to get prices");
    
    assert_eq!(ticks.len(), 3);
    
    for tick in &ticks {
        assert!(instruments.contains(&tick.instrument));
        assert!(tick.bid > 0.0);
        assert!(tick.ask > 0.0);
    }
}

#[tokio::test]
#[ignore]
async fn test_get_candles() {
    let client = get_test_client();
    
    let candles = client.get_candles("EUR_USD", Granularity::M5, 10).await
        .expect("Failed to get candles");
    
    assert_eq!(candles.len(), 10);
    
    for candle in &candles {
        assert_eq!(candle.instrument, "EUR_USD");
        assert!(candle.open > 0.0);
        assert!(candle.high >= candle.open);
        assert!(candle.low <= candle.open);
        assert!(candle.close > 0.0);
        assert!(candle.volume >= 0);
    }
    
    println!("Latest candle: O={} H={} L={} C={}",
        candles.last().unwrap().open,
        candles.last().unwrap().high,
        candles.last().unwrap().low,
        candles.last().unwrap().close
    );
}

#[tokio::test]
#[ignore]
async fn test_get_candles_max_count() {
    let client = get_test_client();
    
    // OANDA allows max 5000 candles
    let result = client.get_candles("EUR_USD", Granularity::M1, 5000).await;
    assert!(result.is_ok());
    
    // Should fail with count > 5000
    let result = client.get_candles("EUR_USD", Granularity::M1, 5001).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_invalid_instrument() {
    let client = get_test_client();
    
    let result = client.get_current_price("INVALID_PAIR").await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_get_instruments() {
    let client = get_test_client();
    
    let instruments = client.get_instruments().await
        .expect("Failed to get instruments");
    
    assert!(!instruments.is_empty());
    
    // Check that EUR_USD is in the list
    let eur_usd = instruments.iter()
        .find(|i| i.name == "EUR_USD");
    
    assert!(eur_usd.is_some(), "EUR_USD should be available");
    
    println!("Total instruments: {}", instruments.len());
}

#[tokio::test]
#[ignore]
async fn test_rate_limiting() {
    let client = get_test_client();
    
    let start = std::time::Instant::now();
    
    // Make 20 rapid requests
    for _ in 0..20 {
        let _ = client.get_current_price("EUR_USD").await;
    }
    
    let elapsed = start.elapsed();
    
    // With 100 req/sec limit, 20 requests should take < 1 second
    assert!(elapsed < Duration::from_secs(1), 
        "Rate limiting seems too restrictive: took {:?}", elapsed);
    
    println!("20 requests completed in {:?}", elapsed);
}

#[tokio::test]
#[ignore]
async fn test_granularity_parsing() {
    use std::str::FromStr;
    
    assert!(Granularity::from_str("M5").is_ok());
    assert!(Granularity::from_str("H1").is_ok());
    assert!(Granularity::from_str("d").is_ok()); // Case insensitive
    assert!(Granularity::from_str("INVALID").is_err());
}

#[tokio::test]
#[ignore]
async fn test_concurrent_requests() {
    let client = get_test_client();
    
    // Make 5 concurrent requests
    let futures: Vec<_> = (0..5)
        .map(|_| client.get_current_price("EUR_USD"))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
    }
}