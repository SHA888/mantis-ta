//! Backtesting engine for strategy validation and performance analysis.
//!
//! This module is planned for v0.3.0 and will provide:
//! - `BacktestConfig` struct for configuration (initial capital, commission, slippage, etc.)
//! - `ExecutionModel` enum (NextBarOpen, CurrentBarClose)
//! - `backtest()` runner function for the main execution loop
//! - `BrokerSim` for simulated order fills with slippage
//! - `Portfolio` for position tracking and cash accounting
//! - `BacktestMetrics` for comprehensive performance analysis
//! - Integrity rules: no lookahead bias, next-bar execution, proper commission/slippage handling
//!
//! See [SPEC.md](../SPEC.md) §6 for detailed requirements.
