//! Data models for OANDA API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OHLCV candle data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    pub instrument: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub complete: bool, // true if candle is finalized
}

/// Real-time tick/quote
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tick {
    pub instrument: String,
    pub timestamp: DateTime<Utc>,
    pub bid: f64,
    pub ask: f64,
}

impl Tick {
    /// Calculate spread
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate mid price
    pub fn mid(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

/// Time granularity for candles
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Granularity {
    #[serde(rename = "S5")]
    S5, // 5 seconds
    #[serde(rename = "S10")]
    S10, // 10 seconds
    #[serde(rename = "S15")]
    S15, // 15 seconds
    #[serde(rename = "S30")]
    S30, // 30 seconds
    #[serde(rename = "M1")]
    M1, // 1 minute
    #[serde(rename = "M2")]
    M2, // 2 minutes
    #[serde(rename = "M5")]
    M5, // 5 minutes
    #[serde(rename = "M15")]
    M15, // 15 minutes
    #[serde(rename = "M30")]
    M30, // 30 minutes
    #[serde(rename = "H1")]
    H1, // 1 hour
    #[serde(rename = "H4")]
    H4, // 4 hours
    #[serde(rename = "D")]
    D, // Daily
    #[serde(rename = "W")]
    W, // Weekly
    #[serde(rename = "M")]
    M, // Monthly
}

impl Granularity {
    /// Get duration in seconds
    pub fn duration_seconds(&self) -> u64 {
        match self {
            Granularity::S5 => 5,
            Granularity::S10 => 10,
            Granularity::S15 => 15,
            Granularity::S30 => 30,
            Granularity::M1 => 60,
            Granularity::M2 => 120,
            Granularity::M5 => 300,
            Granularity::M15 => 900,
            Granularity::M30 => 1800,
            Granularity::H1 => 3600,
            Granularity::H4 => 14400,
            Granularity::D => 86400,
            Granularity::W => 604800,
            Granularity::M => 2592000, // Approximate
        }
    }
}

impl std::fmt::Display for Granularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Granularity::S5 => "S5",
            Granularity::S10 => "S10",
            Granularity::S15 => "S15",
            Granularity::S30 => "S30",
            Granularity::M1 => "M1",
            Granularity::M2 => "M2",
            Granularity::M5 => "M5",
            Granularity::M15 => "M15",
            Granularity::M30 => "M30",
            Granularity::H1 => "H1",
            Granularity::H4 => "H4",
            Granularity::D => "D",
            Granularity::W => "W",
            Granularity::M => "M",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for Granularity {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "S5" => Ok(Granularity::S5),
            "S10" => Ok(Granularity::S10),
            "S15" => Ok(Granularity::S15),
            "S30" => Ok(Granularity::S30),
            "M1" => Ok(Granularity::M1),
            "M2" => Ok(Granularity::M2),
            "M5" => Ok(Granularity::M5),
            "M15" => Ok(Granularity::M15),
            "M30" => Ok(Granularity::M30),
            "H1" => Ok(Granularity::H1),
            "H4" => Ok(Granularity::H4),
            "D" => Ok(Granularity::D),
            "W" => Ok(Granularity::W),
            "M" => Ok(Granularity::M),
            _ => Err(crate::error::Error::InvalidGranularity(s.to_string())),
        }
    }
}

/// Account summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub balance: f64,
    pub nav: f64, // Net Asset Value
    pub unrealized_pl: f64,
    pub realized_pl: f64,
    pub margin_used: f64,
    pub margin_available: f64,
    pub open_trade_count: i32,
    pub open_position_count: i32,
    pub currency: String,
}

/// Instrument information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub display_name: String,
    pub pip_location: i32,
    pub trade_units_precision: i32,
    pub minimum_trade_size: f64,
    pub maximum_trade_size: f64,
    pub margin_rate: f64,
}

/// Internal OANDA API response structures
#[derive(Debug, Deserialize)]
pub(crate) struct CandlesResponse {
    pub instrument: String,
    pub granularity: String,
    pub candles: Vec<OandaCandle>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OandaCandle {
    pub time: String,
    pub volume: i64,
    pub complete: bool,
    pub mid: Option<OandaPriceData>,
    pub bid: Option<OandaPriceData>,
    pub ask: Option<OandaPriceData>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OandaPriceData {
    pub o: String,
    pub h: String,
    pub l: String,
    pub c: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PricingResponse {
    pub prices: Vec<OandaPrice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OandaPrice {
    pub instrument: String,
    pub time: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PriceLevel {
    pub price: String,
    pub liquidity: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AccountResponse {
    pub account: OandaAccount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OandaAccount {
    pub id: String,
    pub balance: String,
    pub nav: String,
    pub unrealized_pl: String,
    pub realized_pl: String,
    pub margin_used: String,
    pub margin_available: String,
    pub open_trade_count: i32,
    pub open_position_count: i32,
    pub currency: String,
}

impl OandaCandle {
    /// Convert to our Candle type
    pub(crate) fn to_candle(&self, instrument: String) -> crate::Result<Candle> {
        let price_data =
            self.mid
                .as_ref()
                .or(self.bid.as_ref())
                .ok_or_else(|| crate::Error::ApiError {
                    code: 0,
                    message: format!("No price data in candle."),
                })?;

        Ok(Candle {
            instrument,
            timestamp: DateTime::parse_from_rfc3339(&self.time)
                .map_err(|e| crate::Error::ApiError {
                    code: 0,
                    message: format!("Failed to parse datetime: {}", e),
                })?
                .with_timezone(&Utc),
            open: price_data.o.parse().unwrap_or(0.0),
            high: price_data.h.parse().unwrap_or(0.0),
            low: price_data.l.parse().unwrap_or(0.0),
            close: price_data.c.parse().unwrap_or(0.0),
            volume: self.volume,
            complete: self.complete,
        })
    }
}

impl OandaPrice {
    /// Convert to our Tick type
    pub(crate) fn to_tick(&self) -> crate::Result<Tick> {
        let bid = self
            .bids
            .first()
            .ok_or_else(|| crate::Error::ApiError {
                code: 0,
                message: format!("No bid data."),
            })?
            .price
            .parse()
            .unwrap_or(0.0);

        let ask = self
            .asks
            .first()
            .ok_or_else(|| crate::Error::ApiError {
                code: 0,
                message: format!("No ask data."),
            })?
            .price
            .parse()
            .unwrap_or(0.0);

        Ok(Tick {
            instrument: self.instrument.clone(),
            timestamp: DateTime::parse_from_rfc3339(&self.time)
                .map_err(|e| crate::Error::ApiError {
                    code: 0,
                    message: format!("Invalid timestamp: {}", e),
                })?
                .with_timezone(&Utc),
            bid,
            ask,
        })
    }
}

impl OandaAccount {
    /// Convert to our AccountSummary type
    pub(crate) fn to_summary(&self) -> AccountSummary {
        AccountSummary {
            id: self.id.clone(),
            balance: self.balance.parse().unwrap_or(0.0),
            nav: self.nav.parse().unwrap_or(0.0),
            unrealized_pl: self.unrealized_pl.parse().unwrap_or(0.0),
            realized_pl: self.realized_pl.parse().unwrap_or(0.0),
            margin_used: self.margin_used.parse().unwrap_or(0.0),
            margin_available: self.margin_available.parse().unwrap_or(0.0),
            open_trade_count: self.open_trade_count,
            open_position_count: self.open_position_count,
            currency: self.currency.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_spread() {
        let tick = Tick {
            instrument: "EUR_USD".to_string(),
            timestamp: Utc::now(),
            bid: 1.1000,
            ask: 1.1002,
        };

        assert!((tick.spread() - 0.0002).abs() < f64::EPSILON);
        assert!((tick.mid() - 1.1001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tick_creation() {
        let tick = Tick {
            instrument: "USD_JPY".to_string(),
            timestamp: Utc::now(),
            bid: 110.50,
            ask: 110.52,
        };
        const FLOAT_TOLERANCE: f64 = 1e-10;

        assert_eq!(tick.instrument, "USD_JPY");
        assert!((tick.spread() - 0.02).abs() < FLOAT_TOLERANCE);
        assert!((tick.mid() - 110.51).abs() < FLOAT_TOLERANCE);
    }

    #[test]
    fn test_granularity_from_str() {
        assert_eq!("M5".parse::<Granularity>().unwrap(), Granularity::M5);
        assert_eq!("h1".parse::<Granularity>().unwrap(), Granularity::H1);
        assert!("INVALID".parse::<Granularity>().is_err());
    }

    #[test]
    fn test_granularity_duration() {
        assert_eq!(Granularity::M5.duration_seconds(), 300);
        assert_eq!(Granularity::H1.duration_seconds(), 3600);
        assert_eq!(Granularity::D.duration_seconds(), 86400);
        assert_eq!(Granularity::W.duration_seconds(), 604800);
    }

    #[test]
    fn test_granularity_display() {
        assert_eq!(Granularity::M5.to_string(), "M5");
        assert_eq!(Granularity::H4.to_string(), "H4");
        assert_eq!(Granularity::D.to_string(), "D");
    }

    #[test]
    fn test_candle_creation() {
        let candle = Candle {
            instrument: "GBP_USD".to_string(),
            timestamp: Utc::now(),
            open: 1.3000,
            high: 1.3010,
            low: 1.2990,
            close: 1.3005,
            volume: 100,
            complete: true,
        };

        assert_eq!(candle.instrument, "GBP_USD");
        assert!(candle.high >= candle.low);
        assert!(candle.complete);
    }
}
