//! Tier 1 indicators with streaming and batch APIs.
//!
//! # Examples
//! Compute several indicators over a generated candle series:
//!
//! ```rust
//! use mantis_ta::indicators::{
//!     BollingerBands, Indicator, MACD, OBV, PivotPoints, RSI, SMA, Stochastic, VolumeSMA, ATR, EMA,
//! };
//! use mantis_ta::types::Candle;
//!
//! // Build a small candle series (60 bars) for the examples.
//! let candles: Vec<Candle> = (0..60)
//!     .map(|i| Candle {
//!         timestamp: i,
//!         open: 100.0 + i as f64 * 0.1,
//!         high: 100.2 + i as f64 * 0.1,
//!         low:  99.8 + i as f64 * 0.1,
//!         close:100.1 + i as f64 * 0.1,
//!         volume: 1_000.0 + i as f64,
//!     })
//!     .collect();
//!
//! // Batch calculations (Vec<Option<_>> aligned to input)
//! let sma = SMA::new(20).calculate(&candles);
//! let ema = EMA::new(20).calculate(&candles);
//! let macd = MACD::new(12, 26, 9).calculate(&candles);
//! let rsi = RSI::new(14).calculate(&candles);
//! let stoch = Stochastic::new(14, 3).calculate(&candles);
//! let bb = BollingerBands::new(20, 2.0).calculate(&candles);
//! let atr = ATR::new(14).calculate(&candles);
//! let vol_sma = VolumeSMA::new(20).calculate(&candles);
//! let obv = OBV::new().calculate(&candles);
//! let pivot = PivotPoints::new().calculate(&candles);
//!
//! // All indicators produce `None` until their warmup is reached.
//! assert!(sma.iter().take(19).all(|v| v.is_none()));
//! assert!(ema.iter().take(19).all(|v| v.is_none()));
//! assert!(rsi.iter().take(13).all(|v| v.is_none()));
//! // Stochastic warmup = k + d - 1 (14 + 3 - 1 = 16)
//! assert!(stoch.iter().take(15).all(|v| v.is_none()));
//! assert!(stoch[15].is_some());
//! assert!(bb.iter().take(19).all(|v| v.is_none()));
//! assert!(atr.iter().take(13).all(|v| v.is_none()));
//! assert!(macd.iter().all(|v| v.is_none()) || macd.iter().any(|v| v.is_some()));
//! assert!(pivot.iter().all(|v| v.is_some()));
//! assert!(obv.iter().all(|v| v.is_some()));
//! assert!(vol_sma.iter().take(19).all(|v| v.is_none()));
//!
//! // Streaming example: re-use the same indicator instance per-bar
//! let mut rsi_stream = RSI::new(14);
//! for c in &candles {
//!     let _ = rsi_stream.next(c);
//! }
//! ```

use crate::types::Candle;
use std::fmt::Debug;

pub mod momentum;
pub mod obv;
pub mod support_resistance;
pub mod trend;
pub mod volatility;
pub mod volume;
pub use momentum::{CCI, ROC, RSI, Stochastic, WilliamsR};
pub use obv::OBV;
pub use support_resistance::PivotPoints;
pub use trend::{ADX, DEMA, EMA, MACD, SMA, TEMA, WMA};
pub use volatility::{ATR, BollingerBands, StdDev};
pub use volume::VolumeSMA;

/// Core interface for all streaming technical indicators.
///
/// Implementations should be stateful and cheap to `next()`. The default
/// `calculate` helper performs a streaming pass over a slice of candles.
pub trait Indicator: Send + Sync {
    /// Concrete output type for this indicator (e.g., `f64` or a struct).
    type Output: Clone + Debug;

    /// Feed one candle; returns `Some(output)` once warmup is satisfied, otherwise `None`.
    fn next(&mut self, candle: &Candle) -> Option<Self::Output>;

    /// Reset to initial state.
    fn reset(&mut self);

    /// Number of candles required before the first valid output.
    fn warmup_period(&self) -> usize;

    /// Batch compute over a candle series (default streaming loop).
    fn calculate(&self, candles: &[Candle]) -> Vec<Option<Self::Output>> {
        let mut instance = self.clone_boxed();
        candles.iter().map(|c| instance.next(c)).collect()
    }

    /// Clone into a boxed trait object.
    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>>;
}

/// Extension of [`Indicator`] that supports state snapshot/restore.
pub trait IncrementalIndicator: Indicator {
    /// Serializable (or cloneable) state representation.
    type State: Clone;

    /// Snapshot internal state for checkpointing.
    fn state(&self) -> Self::State;

    /// Restore from a prior snapshot.
    fn restore(&mut self, state: Self::State);
}

// Re-export common types for convenience.
pub use crate::types::{BollingerOutput, MacdOutput, PivotOutput, StochasticOutput};
