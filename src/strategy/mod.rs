//! Strategy composition engine for rule-based trading signal generation.
//!
//! This module provides:
//! - `Condition` struct for composing indicator-based rules
//! - `CompareTarget` enum (Value, Indicator, Scaled) for flexible comparisons
//! - `Operator` enum (CrossesAbove, CrossesBelow, IsAbove, IsBelow, IsBetween, etc.)
//! - `ConditionGroup` enum (AllOf, AnyOf) for logical grouping
//! - `IndicatorRef` type with convenience constructors (`sma()`, `ema()`, `rsi()`, etc.)
//! - `Strategy` struct with fluent builder API
//! - Builder methods: `timeframe`, `entry`, `exit`, `stop_loss`, `take_profit`, position sizing, risk limits
//! - Strategy validation at build time
//! - JSON serialization/deserialization (behind `serde` feature)
//!
//! Batch and streaming evaluation modes are planned for v0.3.0.
//! See [SPEC.md](../SPEC.md) §5 for detailed requirements.

pub mod indicator_ref;
pub mod types;

pub use indicator_ref::{all_of, any_of, IndicatorRef, ScaledIndicatorRef};
pub use types::{
    CompareTarget, Condition, ConditionGroup, ConditionNode, Operator, StopLoss, Strategy,
    StrategyBuilder, TakeProfit,
};
