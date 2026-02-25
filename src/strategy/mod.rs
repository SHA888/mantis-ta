//! Strategy composition engine for rule-based trading signal generation.
//!
//! This module is planned for v0.2.0 and will provide:
//! - `Condition` struct for composing indicator-based rules
//! - `CompareTarget` enum (Value, Indicator, Scaled) for flexible comparisons
//! - `Operator` enum (CrossesAbove, CrossesBelow, IsAbove, IsBelow, IsBetween, etc.)
//! - `ConditionGroup` enum (AllOf, AnyOf) for logical grouping
//! - `IndicatorRef` type with convenience constructors (`sma()`, `ema()`, `rsi()`, etc.)
//! - `Strategy` struct with fluent builder API
//! - Builder methods: `timeframe`, `entry`, `exit`, `stop_loss`, `take_profit`, position sizing, risk limits
//! - Strategy validation at build time
//! - Batch evaluation mode (`strategy.evaluate(&candles)`)
//! - Streaming evaluation mode (`strategy.into_engine()` + `engine.next(&candle)`)
//! - JSON serialization/deserialization (behind `serde` feature)
//!
//! See [SPEC.md](../SPEC.md) §5 for detailed requirements.
