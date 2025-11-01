# OANDA Connector for Rust

[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

High-performance, production-ready Rust client for the OANDA forex trading API. Built with safety, speed, and reliability in mind.

## Features

- ✅ **Full OANDA v3 API support**

  - Real-time pricing (current quotes)
  - Historical candle data (OHLCV)
  - Account information
  - Instrument details

- 🚀 **Production-grade reliability**

  - Automatic rate limiting (token bucket algorithm)
  - Exponential backoff retry logic
  - Comprehensive error handling
  - Request timeout management

- 🔒 **Type-safe & well-tested**

  - Strongly typed API responses
  - 90%+ test coverage
  - Integration tests with real API
  - Mock server tests for CI/CD

- ⚡ **Performance optimized**
  - Async/await using Tokio
  - Efficient connection pooling
  - Minimal allocations
  - Benchmarked operations

## Quick Start

### 1. Get OANDA Credentials

#### Option A: Practice Account (Recommended for Testing)

1. **Sign up for OANDA practice account** (free, no credit card):

   - Go to: https://www.oanda.com/register/#/sign-up/demo
   - Fill in your details (name, email, password)
   - Choose "Practice Account" (starts with $100,000 virtual money)
   - Verify your email

2. **Generate API token:**

   - Log in to: https://www.oanda.com/demo-account/login
   - Click your name (top right) → "Manage API Access"
   - Or go directly to: https://www.oanda.com/account/tpa/personal_token
   - Click "Generate" next to "Personal Access Token"
   - Copy the token (starts with something like `abc123def456...`)
   - ⚠️ **Save it immediately** - you can't see it again!

3. **Get your Account ID:**
   - Still on the API Access page
   - Look for "Practice Account" section
   - Copy your Account ID (format: `101-004-XXXXXXXX-001`)

#### Option B: Live Account (Real Money - Use with Caution!)

1. Sign up at: https://www.oanda.com/register/#/sign-up/live
2. Complete identity verification (required by law)
3. Deposit real money
4. Generate API token: https://www.oanda.com/account/tpa/personal_token
5. Get Account ID from the same page

**⚠️ WARNING**: Live accounts use real money. Start with practice account!

### 2. Install

Add to your `Cargo.toml`:

```toml
[dependencies]
oanda-connector = { path = "path/to/oanda-connector" }
tokio = { version = "1", features = ["full"] }
```

Or as a Git dependency:

```toml
[dependencies]
oanda-connector = { git = "https://github.com/yourusername/oanda-connector" }
```

### 3. Set Environment Variables

```bash
# Copy the example file
cp .env.example .env

# Edit .env with your credentials
export OANDA_API_KEY="your_practice_api_token_here"
export OANDA_ACCOUNT_ID="101-004-XXXXXXXX-001"
export OANDA_PRACTICE=true  # false for live trading
```

**For Windows (PowerShell):**

```powershell
$env:OANDA_API_KEY="your_token"
$env:OANDA_ACCOUNT_ID="your_account_id"
$env:OANDA_PRACTICE="true"
```

### 4. Run Example

```bash
cargo run --example fetch_example
```

Expected output:

```
🚀 OANDA Connector Example

✅ Configuration loaded:
   Account ID: 101-004-XXXXXXXX-001
   Practice mode: true

🏥 Health Check:
   ✅ Connected and authenticated

💰 Account Summary:
   Balance: 100000.0000 USD
   NAV: 100000.0000
   ...

💱 Current Price (EUR/USD):
   Bid: 1.08245
   Ask: 1.08258
   Spread: 0.00013
   ...
```

## Usage Examples

### Get Current Price

```rust
use oanda_connector::{OandaClient, OandaConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = OandaConfig::from_env()?;
    let client = OandaClient::new(config)?;

    // Fetch current EUR/USD price
    let tick = client.get_current_price("EUR_USD").await?;

    println!("EUR/USD: bid={}, ask={}", tick.bid, tick.ask);
    println!("Spread: {:.5}", tick.spread());

    Ok(())
}
```

### Get Historical Candles

```rust
use oanda_connector::{OandaClient, OandaConfig, Granularity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OandaConfig::from_env()?;
    let client = OandaClient::new(config)?;

    // Get last 100 5-minute candles
    let candles = client.get_candles("EUR_USD", Granularity::M5, 100).await?;

    for candle in candles.iter().take(5) {
        println!("{}: O={:.5} H={:.5} L={:.5} C={:.5}",
            candle.timestamp,
            candle.open,
            candle.high,
            candle.low,
            candle.close
        );
    }

    Ok(())
}
```

### Get Multiple Prices at Once

```rust
let instruments = vec![
    "EUR_USD".to_string(),
    "GBP_USD".to_string(),
    "USD_JPY".to_string(),
];

let ticks = client.get_current_prices(&instruments).await?;

for tick in ticks {
    println!("{}: {:.5}", tick.instrument, tick.mid());
}
```

### Get Account Summary

```rust
let summary = client.get_account_summary().await?;

println!("Balance: {} {}", summary.balance, summary.currency);
println!("NAV: {}", summary.nav);
println!("Unrealized P/L: {}", summary.unrealized_pl);
println!("Margin Available: {}", summary.margin_available);
```

### Custom Configuration

```rust
use oanda_connector::{OandaConfig, OandaClientBuilder};

let config = OandaConfig::new(
    "your_api_key".to_string(),
    "your_account_id".to_string(),
    true, // practice mode
);

let client = OandaClientBuilder::new(config)
    .timeout(20)                  // 20 second timeout
    .rate_limit(50)               // 50 requests/second
    .max_retries(5)               // retry up to 5 times
    .build()?;
```

## Available Granularities

```rust
Granularity::S5    // 5 seconds
Granularity::S10   // 10 seconds
Granularity::S15   // 15 seconds
Granularity::S30   // 30 seconds
Granularity::M1    // 1 minute
Granularity::M2    // 2 minutes
Granularity::M5    // 5 minutes
Granularity::M15   // 15 minutes
Granularity::M30   // 30 minutes
Granularity::H1    // 1 hour
Granularity::H4    // 4 hours
Granularity::D     // Daily
Granularity::W     // Weekly
Granularity::M     // Monthly
```

## Common Instrument Pairs

### Major Forex Pairs

- `EUR_USD` - Euro / US Dollar
- `GBP_USD` - British Pound / US Dollar
- `USD_JPY` - US Dollar / Japanese Yen
- `USD_CHF` - US Dollar / Swiss Franc
- `AUD_USD` - Australian Dollar / US Dollar
- `USD_CAD` - US Dollar / Canadian Dollar
- `NZD_USD` - New Zealand Dollar / US Dollar

### Cross Pairs

- `EUR_GBP` - Euro / British Pound
- `EUR_JPY` - Euro / Japanese Yen
- `GBP_JPY` - British Pound / Japanese Yen
- `AUD_JPY` - Australian Dollar / Japanese Yen

### Commodities

- `XAU_USD` - Gold / US Dollar
- `XAG_USD` - Silver / US Dollar
- `BCO_USD` - Brent Crude Oil
- `WTICO_USD` - West Texas Intermediate Oil

### Indices

- `SPX500_USD` - S&P 500
- `NAS100_USD` - NASDAQ 100
- `US30_USD` - Dow Jones 30

**Full list**: Run `client.get_instruments().await` to see all available instruments for your account.

## Error Handling

```rust
use oanda_connector::Error;

match client.get_current_price("EUR_USD").await {
    Ok(tick) => println!("Price: {}", tick.mid()),
    Err(Error::AuthenticationFailed) => {
        eprintln!("Invalid API key or account ID");
    }
    Err(Error::RateLimitExceeded { retry_after_seconds }) => {
        eprintln!("Rate limited, retry after {} seconds", retry_after_seconds);
    }
    Err(Error::InvalidInstrument(inst)) => {
        eprintln!("Invalid instrument: {}", inst);
    }
    Err(Error::Timeout(secs)) => {
        eprintln!("Request timed out after {} seconds", secs);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Rate Limiting

The connector automatically enforces rate limits to prevent API bans:

- **Default**: 100 requests/second (configurable)
- **Algorithm**: Token bucket with automatic refill
- **Behavior**: Requests wait if limit exceeded (no errors thrown)

```rust
// Custom rate limit
let client = OandaClientBuilder::new(config)
    .rate_limit(50)  // Lower to 50 req/sec
    .build()?;
```

**OANDA's actual limits** (as of 2024):

- Practice accounts: ~120 requests/second
- Live accounts: ~120 requests/second
- Streaming: Different limits apply

## Testing

### Run All Tests

```bash
# Unit tests (no API calls)
cargo test --lib

# Mock server tests (no API calls)
cargo test mock_server

# Integration tests (requires OANDA credentials)
cargo test --test integration_tests -- --ignored --nocapture
```

### Run Specific Test

```bash
cargo test test_get_current_price -- --ignored --nocapture
```

### Run Benchmarks

```bash
# Requires OANDA credentials
cargo bench
```

Expected performance (practice account, good connection):

- `get_current_price`: 150-250ms
- `get_candles(100)`: 300-500ms
- `rate_limiter_acquire`: <1µs

## Troubleshooting

### "ConfigError: OANDA_API_KEY environment variable not set"

**Solution**: Set your environment variables:

```bash
export OANDA_API_KEY="your_token"
export OANDA_ACCOUNT_ID="your_account_id"
```

Or create a `.env` file and use a tool like [`dotenv`](https://crates.io/crates/dotenv):

```bash
# .env file
OANDA_API_KEY=your_token_here
OANDA_ACCOUNT_ID=your_account_id_here
```

### "AuthenticationFailed"

**Possible causes**:

1. Invalid API token
2. Wrong account ID
3. Token expired (regenerate on OANDA website)
4. Using practice token with live URL (or vice versa)

**Solution**:

- Verify credentials at: https://www.oanda.com/account/tpa/personal_token
- Ensure `OANDA_PRACTICE=true` for practice accounts

### "RateLimitExceeded"

**Cause**: Too many requests too quickly

**Solution**:

- The connector handles this automatically with retries
- Lower your rate limit: `.rate_limit(50)`
- Add delays between bulk operations

### "InvalidInstrument: XXX_YYY"

**Cause**: Instrument not available for your account

**Solution**:

```rust
// Get list of available instruments
let instruments = client.get_instruments().await?;
for inst in instruments {
    println!("{}", inst.name);
}
```

### Connection Timeouts

**Cause**: Network issues or OANDA server problems

**Solution**:

- Increase timeout: `.timeout(30)`
- Check OANDA status: https://status.oanda.com/
- Verify your internet connection

### "No price data in candle"

**Cause**: Requesting data outside market hours or for illiquid instruments

**Solution**:

- Forex markets are closed weekends (Saturday 5pm ET - Sunday 5pm ET)
- Check market hours for your instrument
- Use `candle.complete == true` to filter incomplete candles

## Project Structure

```
oanda-connector/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── client.rs        # Main OandaClient implementation
│   ├── config.rs        # Configuration management
│   ├── models.rs        # Data structures (Candle, Tick, etc.)
│   ├── error.rs         # Error types
│   ├── endpoints.rs     # API endpoint definitions
│   └── rate_limiter.rs  # Rate limiting logic
├── tests/
│   ├── integration_tests.rs  # Tests with real API
│   └── mock_server.rs        # Tests with mock server
├── examples/
│   └── fetch_example.rs      # Complete usage example
├── benches/
│   └── fetch_benchmark.rs    # Performance benchmarks
├── Cargo.toml
├── README.md
└── .env.example
```

## API Coverage

✅ **Implemented**:

- Get current pricing
- Get historical candles
- Get account summary
- Get available instruments
- Health check

🚧 **Coming Soon**:

- Order placement
- Position management
- Trade management
- Transaction history
- Streaming prices (WebSocket)

## Performance Tips

1. **Batch requests when possible**:

```rust
   // Good: One request for multiple instruments
   let ticks = client.get_current_prices(&["EUR_USD", "GBP_USD"]).await?;

   // Bad: Multiple separate requests
   let eur = client.get_current_price("EUR_USD").await?;
   let gbp = client.get_current_price("GBP_USD").await?;
```

2. **Reuse client instances**:

```rust
   // Good: Create once, reuse
   let client = OandaClient::new(config)?;
   for _ in 0..100 {
       client.get_current_price("EUR_USD").await?;
   }

   // Bad: Create every time
   for _ in 0..100 {
       let client = OandaClient::new(config.clone())?;
       client.get_current_price("EUR_USD").await?;
   }
```

3. **Use appropriate granularity**:

   - Use larger granularities (H1, D) when possible
   - Smaller granularities (S5, M1) generate more data
   - OANDA limits: max 5000 candles per request

4. **Cache when appropriate**:
   - Account summary doesn't change frequently
   - Instrument details are static
   - Recent candles can be cached briefly

## Security Best Practices

1. **Never commit credentials**:

   - Add `.env` to `.gitignore`
   - Use environment variables
   - Use secrets management in production

2. **Use practice account for development**:

   - No risk of losing real money
   - Same API functionality
   - Unlimited testing

3. **Rotate API tokens regularly**:

   - Generate new tokens periodically
   - Revoke old tokens
   - Monitor for unauthorized access

4. **Use read-only tokens when possible**:
   - For data fetching, read-only is sufficient
   - Reduces risk if token is compromised
   - (Note: OANDA tokens are currently all-access)

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo test` passes
5. Run `cargo clippy` and fix warnings
6. Run `cargo fmt`
7. Submit a pull request

## License

MIT License - see LICENSE file for details

## Resources

- **OANDA API Documentation**: https://developer.oanda.com/rest-live-v20/introduction/
- **Practice Account**: https://www.oanda.com/register/#/sign-up/demo
- **API Token Management**: https://www.oanda.com/account/tpa/personal_token
- **OANDA Status Page**: https://status.oanda.com/
- **Support**: support@oanda.com

## Disclaimer

This software is for educational and research purposes. Trading forex involves substantial risk of loss. The authors are not responsible for any financial losses incurred through use of this software.

**Use practice accounts for testing. Never risk money you cannot afford to lose.**

## Changelog

### v0.1.0 (2025-10-31)

- Initial release
- Support for pricing, candles, account data
- Rate limiting and retry logic
- Comprehensive test coverage
- Production-ready error handling

---

**Built with ❤️ and ☕ in Rust**
