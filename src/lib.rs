pub mod indicators;
pub mod prelude;
pub mod types;

#[cfg(feature = "strategy")]
pub mod strategy;

// Placeholder module for future features (v0.3.0+)
#[cfg(feature = "backtest")]
pub mod backtest;

// Internal utilities
pub(crate) mod utils;
