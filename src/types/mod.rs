use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single OHLCV bar.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub timestamp: i64, // Unix timestamp ms
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Which price field to use as input source.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceSource {
    Open,
    High,
    Low,
    Close,
    HLC3,
    OHLC4,
    HL2,
}

/// Trading direction.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

/// Signal emitted by strategy evaluation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Entry(Side),
    Exit(ExitReason),
    Hold,
}

/// Why an exit was triggered.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ExitReason {
    RuleTriggered,
    StopLoss,
    TakeProfit,
    TrailingStop,
    DailyLossLimit,
    DrawdownBreaker,
}

/// Candle timeframe.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
    MN1,
}

/// MACD output values.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacdOutput {
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
}

/// Bollinger Bands output values.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BollingerOutput {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

/// Stochastic oscillator output values.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StochasticOutput {
    pub k: f64,
    pub d: f64,
}

/// Pivot points output values.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PivotOutput {
    pub pp: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

/// Common error type for the crate.
#[derive(Error, Debug)]
pub enum MantisError {
    #[error("Invalid parameter: {param} = {value} ({reason})")]
    InvalidParameter {
        param: &'static str,
        value: String,
        reason: &'static str,
    },

    #[error("Insufficient data: need {required} candles, got {provided}")]
    InsufficientData { required: usize, provided: usize },

    #[error("Strategy validation failed: {0}")]
    StrategyValidation(String),

    #[error("Backtest error: {0}")]
    BacktestError(String),
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, MantisError>;
