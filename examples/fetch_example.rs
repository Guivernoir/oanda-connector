//! Example: Fetch current prices and historical candles from OANDA
//! 
//! Usage:
//!   export OANDA_API_KEY="your_key"
//!   export OANDA_ACCOUNT_ID="your_id"
//!   cargo run --example fetch_example

use oanda_connector::{OandaClient, OandaConfig, Granularity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 OANDA Connector Example\n");
    
    // Load configuration from environment
    let config = OandaConfig::from_env()?;
    println!("✅ Configuration loaded:");
    println!("   Account ID: {}", config.account_id);
    println!("   Practice mode: {}", config.practice);
    println!("   Base URL: {}\n", config.get_base_url());
    
    // Create client
    let client = OandaClient::new(config)?;
    println!("✅ Client created\n");
    
    // 1. Health check
    println!("🏥 Health Check:");
    match client.health_check().await {
        Ok(true) => println!("   ✅ Connected and authenticated\n"),
        Ok(false) => {
            println!("   ❌ Authentication failed\n");
            return Ok(());
        }
        Err(e) => {
            println!("   ❌ Connection failed: {}\n", e);
            return Ok(());
        }
    }
    
    // 2. Get account summary
    println!("💰 Account Summary:");
    match client.get_account_summary().await {
        Ok(summary) => {
            println!("   Balance: {} {}", summary.balance, summary.currency);
            println!("   NAV: {}", summary.nav);
            println!("   Unrealized P/L: {}", summary.unrealized_pl);
            println!("   Margin Used: {}", summary.margin_used);
            println!("   Margin Available: {}", summary.margin_available);
            println!("   Open Trades: {}", summary.open_trade_count);
            println!("   Open Positions: {}\n", summary.open_position_count);
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 3. Get current price for EUR/USD
    println!("💱 Current Price (EUR/USD):");
    match client.get_current_price("EUR_USD").await {
        Ok(tick) => {
            println!("   Bid: {}", tick.bid);
            println!("   Ask: {}", tick.ask);
            println!("   Spread: {:.5}", tick.spread());
            println!("   Mid: {}", tick.mid());
            println!("   Time: {}\n", tick.timestamp);
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 4. Get multiple prices at once
    println!("💱 Multiple Prices:");
    let instruments = vec![
        "EUR_USD".to_string(),
        "GBP_USD".to_string(),
        "USD_JPY".to_string(),
    ];
    
    match client.get_current_prices(&instruments).await {
        Ok(ticks) => {
            for tick in ticks {
                println!("   {}: bid={:.5}, ask={:.5}, spread={:.5}",
                    tick.instrument, tick.bid, tick.ask, tick.spread());
            }
            println!();
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 5. Get historical candles
    println!("📊 Historical Candles (EUR/USD, M5, last 10):");
    match client.get_candles("EUR_USD", Granularity::M5, 10).await {
        Ok(candles) => {
            println!("   Fetched {} candles", candles.len());
            for (i, candle) in candles.iter().take(5).enumerate() {
                println!("   [{}] {} | O:{:.5} H:{:.5} L:{:.5} C:{:.5} V:{}",
                    i + 1,
                    candle.timestamp.format("%Y-%m-%d %H:%M"),
                    candle.open,
                    candle.high,
                    candle.low,
                    candle.close,
                    candle.volume
                );
            }
            if candles.len() > 5 {
                println!("   ... ({} more)", candles.len() - 5);
            }
            println!();
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    // 6. Get available instruments (first 10)
    println!("🎯 Available Instruments (first 10):");
    match client.get_instruments().await {
        Ok(instruments) => {
            for (i, instrument) in instruments.iter().take(10).enumerate() {
                println!("   [{}] {} - {}",
                    i + 1,
                    instrument.name,
                    instrument.display_name
                );
            }
            println!("   ... (total: {} instruments)\n", instruments.len());
        }
        Err(e) => println!("   ❌ Error: {}\n", e),
    }
    
    println!("✅ Example completed successfully!");
    
    Ok(())
}