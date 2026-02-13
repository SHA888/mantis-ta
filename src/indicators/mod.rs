use crate::types::Candle;
use std::fmt::Debug;

pub mod trend;
pub use trend::{EMA, SMA};

pub trait Indicator: Send + Sync {
    type Output: Clone + Debug;

    /// Feed one candle; returns current value or None during warmup.
    fn next(&mut self, candle: &Candle) -> Option<Self::Output>;

    /// Reset to initial state.
    fn reset(&mut self);

    /// Candles needed before first valid output.
    fn warmup_period(&self) -> usize;

    /// Batch compute over a candle series (default streaming loop).
    fn calculate(&self, candles: &[Candle]) -> Vec<Option<Self::Output>> {
        let mut instance = self.clone_boxed();
        candles.iter().map(|c| instance.next(c)).collect()
    }

    /// Clone into a boxed trait object.
    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>>;
}

pub trait IncrementalIndicator: Indicator {
    type State: Clone;

    /// Snapshot internal state for checkpointing.
    fn state(&self) -> Self::State;

    /// Restore from a prior snapshot.
    fn restore(&mut self, state: Self::State);
}

// Re-export common types for convenience.
pub use crate::types::{BollingerOutput, MacdOutput, PivotOutput, StochasticOutput};
