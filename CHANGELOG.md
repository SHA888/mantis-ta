# Changelog

All notable changes to this project will be documented in this file.

## [0.5.3] — 2026-03-25

### Changed
- **Rust upgrade**: Updated MSRV from 1.86 to 1.88 for cargo-tarpaulin compatibility
- **CI toolchain**: Upgraded GitHub Actions to use Rust 1.88
- **Code coverage**: Added cargo-tarpaulin integration with Codecov badge
- **Dependencies**: Updated to latest criterion 0.8.2 (requires Rust 1.88)

### Fixed
- **Clippy warnings**: Fixed 33 clippy warnings for Rust 1.88 compatibility
  - Fixed `uninlined_format_args` warnings in indicator_ref.rs and types.rs
  - Fixed `collapsible_if` warnings in evaluator.rs
- **Formatting**: Applied rustfmt to fix line breaks in collapsed if-let statements
- **cargo-tarpaulin**: Fixed dependency conflicts with cargo-platform 0.3.2 and gimli 0.33.0

### Added
- **Code coverage badge**: Added Codecov coverage badge to README (74.92% coverage)
- **Coverage integration**: Full CI coverage pipeline with XML report generation

## [0.5.2] — 2026-03-25

### Changed
- **Rust upgrade**: Updated MSRV from 1.85 to 1.86 for criterion 0.8.2 compatibility
- **Dependencies**: Upgraded criterion to 0.8.2 with latest benchmarking features

## [0.5.1] — 2026-03-25

### Changed
- **Rust edition**: Upgraded from edition 2021 to 2024
- **MSRV**: Updated minimum supported Rust version to 1.85

### Added
- **Dependabot**: Comprehensive dependency monitoring for Rust crates and GitHub Actions
- **Security policy**: SECURITY.md with vulnerability reporting guidelines
- **Issue templates**: Security vulnerability reporting template

## [0.5.0] — 2026-03-03

### Added

#### Tier 2 Indicators: Batch A (8 indicators)
- **ADX** (Average Directional Index) — trend strength measurement with +DI, -DI, ADX values
- **WMA** (Weighted Moving Average) — linearly weighted moving average
- **DEMA** (Double Exponential Moving Average) — reduced-lag moving average
- **TEMA** (Triple Exponential Moving Average) — further reduced-lag moving average
- **CCI** (Commodity Channel Index) — deviation from typical price
- **Williams %R** (Williams Percent Range) — momentum oscillator (-100 to 0)
- **ROC** (Rate of Change) — percentage price change over period
- **StdDev** (Standard Deviation) — volatility measurement

#### Strategy Integration
- `IndicatorRef` convenience constructors for all Batch A indicators (`adx()`, `wma()`, `dema()`, `tema()`, `cci()`, `williams_r()`, `roc()`, `stddev()`)
- Updated `StrategyEngine` evaluator to support all Batch A indicators in strategy builder
- Comprehensive integration test verifying all indicators work in strategy builder → eval → signal flow

#### Testing & Verification
- TA-Lib reference JSON files for all Batch A indicators
- Verification test suite with 10 tests covering consistency, streaming vs batch equivalence, reset functionality
- Comprehensive unit tests for each indicator (edge cases, warmup periods, reset behavior)
- Streaming + batch benchmarks for all 8 indicators (16 total benchmarks)

### Fixed
- **ROC reset bug**: Fixed window size inconsistency in `reset()` method (was `period + 1`, now `period`)
- **CCI performance**: Removed unnecessary `Vec<f64>` allocation on every update, now calculates directly from iterator

### Changed
- Version bump from 0.2.0 to 0.5.0 (v0.4.0 Backtesting Engine released separately)

## [0.4.0] — 2026-03-01

### Added

#### Backtesting Engine
- `BacktestConfig` struct (initial capital, commission, slippage, execution model, fractional shares, margin)
- `ExecutionModel` enum (`NextBarOpen`, `CurrentBarClose`)
- `backtest()` runner function — main execution loop with full trade simulation
- `BrokerSim` — simulated broker with fill simulation and slippage modeling
- `Portfolio` — portfolio state tracking, cash accounting, position sizing

#### Metrics & Reporting
- `BacktestMetrics` struct with comprehensive fields (returns, risk-adjusted metrics, drawdown, trade stats, stress metrics, exposure)
- Metrics calculation from trade history
- Trade log output: entry/exit timestamps, prices, P&L, exit reason, holding period

#### Integrity & Safety
- No lookahead bias: indicators see only data up to current bar
- Next-bar execution: entries/exits fill at next bar's open (default)
- Slippage modeling (configurable, default 0.1% equities, 0.05% forex)
- Commission modeling (flat fee or percentage)
- Cash accounting: cannot buy more than available cash (no hidden margin)
- Overfitting safeguards: walk-forward validation, parameter sensitivity analysis

#### Testing & Benchmarks
- Edge case tests: strategy that never trades, strategy that's always in position
- Full backtest benchmarks: 2yr daily, 1 instrument (target: < 5 ms), 10 instruments (target: < 50 ms)

#### Documentation
- `examples/backtest_momentum.rs` — full backtest with metrics output
- Rustdoc for all backtest public types

## [0.3.0] — 2026-02-28

### Added

#### Strategy Evaluation Engine
- Batch mode: `strategy.evaluate(&candles) -> Vec<Signal>`
- Streaming mode: `strategy.into_engine()` + `engine.next(&candle) -> Signal`
- Condition evaluator: resolve `IndicatorRef` to computed values, apply `Operator`
- Cross-detection state management (previous bar values for CrossesAbove/Below)
- Warmup handling: return `Signal::Hold` until all indicators have sufficient data

#### Testing
- Integration tests: builder → eval → signal accuracy (basic scenarios)
- Golden Cross strategy: verify entry/exit signals against manual calculation
- RSI Mean Reversion strategy: verify signals at known oversold/overbought points
- Edge case tests: single condition, maximum conditions, nested groups
- Streaming vs. batch equivalence: same candles produce same signals in both modes

#### Benchmarks
- Strategy evaluation benchmark: 5 conditions, 2000 bars (target: < 200 µs)
- CI step: run Criterion benchmarks (report only, no gate)

#### Documentation
- Updated `examples/golden_cross_strategy.rs` — now includes evaluation + signal output

## [0.2.0] — 2026-02-26

### Added

#### Strategy Composition Engine
- `Condition` struct for composing indicator-based rules (left indicator, operator, right target)
- `CompareTarget` enum (`Value`, `Indicator`, `Scaled`, `Range`, `None`) for flexible comparisons
- `Operator` enum (`CrossesAbove`, `CrossesBelow`, `IsAbove`, `IsBelow`, `IsBetween`, `Equals`, `IsRising`, `IsFalling`)
- `ConditionGroup` enum (`AllOf`, `AnyOf`) for logical grouping
- `ConditionNode` enum for composable condition trees
- `StopLoss` type (`FixedPercent`, `AtrMultiple`, `Trailing`)
- `TakeProfit` type (`FixedPercent`, `AtrMultiple`)
- `Strategy` struct with fluent builder API and build-time validation
- `IndicatorRef` type with convenience constructors (`sma()`, `ema()`, `rsi()`, `macd()`, `atr()`, `bb_upper()`, etc.)
- `IndicatorRef` condition methods: `crosses_above`, `crosses_below`, `is_above`, `is_below`, `is_between`, `is_rising`, `is_falling`, `equals`, `scaled`
- `ScaledIndicatorRef` for scaled indicator comparisons
- `all_of()` and `any_of()` condition grouping helpers
- Builder validation: required entry/exit/stop-loss/take-profit, parameter range checks, nesting depth and group size limits (SPEC §5.3)
- `PartialEq` derives on all strategy types for testability

#### Feature Flags
- `strategy` feature flag (included in default features) gating the strategy module
- `backtest` feature flag gating the backtest placeholder module
- `required-features` on bench and example targets

#### Serialization
- Full serde support for all strategy types (behind `serde` feature)
- Verified JSON round-trip serialization

#### Documentation
- `examples/golden_cross_strategy.rs` — build a strategy with SMA crossover, serialize to JSON
- Rustdoc for all new public types and methods

#### Testing
- Unit tests for all `Operator` variants
- Unit tests for `ConditionGroup` nesting (AllOf containing AnyOf, depth limits)
- Builder validation tests (missing fields, out-of-range parameters, oversized groups)
- Round-trip serialization tests
- `ScaledIndicatorRef` semantic correctness tests

### Changed
- Version bump from 0.1.1 to 0.2.0
- Strategy and backtest modules now gated behind feature flags

## [0.1.1] — 2026-02-25

### Fixed
- Remove `.unwrap()` panic paths in EMA and RSI indicator hot paths, aligning with SPEC §4.2 "No panics" contract

### Added
- `CLAUDE.md` for Claude Code guidance

## [0.1.0] — 2026-02-14

### Added

#### Tier 1 Indicators (10)
- **SMA** (Simple Moving Average) — streaming and batch computation
- **EMA** (Exponential Moving Average) — SMA-seeded warmup
- **MACD** (Moving Average Convergence Divergence) — line, signal, histogram
- **RSI** (Relative Strength Index) — Wilder smoothing
- **Stochastic Oscillator** — %K and %D lines
- **Bollinger Bands** — upper, middle (SMA), lower bands
- **ATR** (Average True Range) — Wilder smoothing
- **Volume SMA** — volume-weighted moving average
- **OBV** (On-Balance Volume) — cumulative volume indicator
- **Pivot Points** — classic floor-trader levels (PP, R1-3, S1-3)

#### Core Infrastructure
- `Indicator` trait with `next()`, `reset()`, `warmup_period()`, `calculate()`, `clone_boxed()`
- `IncrementalIndicator` trait for state snapshot/restore
- `Candle` struct (OHLCV) with serde support
- Core types: `PriceSource`, `Side`, `Signal`, `ExitReason`, `Timeframe`, `MantisError`
- Output types: `MacdOutput`, `BollingerOutput`, `StochasticOutput`, `PivotOutput`
- Utility modules: `RingBuf` (fixed-size ring buffer), math helpers, crossover detection

#### Testing & Verification
- TA-Lib reference data generator (`fixtures/generate_references.py`)
- Verification tests for all Tier 1 indicators (< 1e-10 relative error vs TA-Lib)
- Unit tests: edge cases, NaN handling, warmup, reset
- Property-based tests: RSI bounds, Bollinger middle = SMA, streaming = batch
- Fuzz tests: random candles never panic, extreme values handled
- 2000-bar synthetic market data fixtures (SPY 5y, AAPL 2y, EURUSD 1h)

#### Benchmarks
- Criterion harness for indicators, strategy_eval, backtest
- Streaming per-bar benchmarks (target: < 100 ns per indicator)
- Batch 2000-bar benchmarks (per SPEC §8 targets)
- Shared `benches/common.rs` helper for candle loading

#### Documentation
- Rustdoc for all public types, traits, and functions
- Runnable doc examples for each Tier 1 indicator
- `examples/basic_indicators.rs` — batch and streaming usage
- `examples/streaming_ema.rs` — per-bar EMA updates
- README with quick-start, feature list, design principles, performance targets
- CI/CD: GitHub Actions workflow (fmt, clippy, test)

### Notes
- All indicators verified against TA-Lib reference outputs
- Zero allocations in streaming `next()` calls
- Safe Rust by default (no unsafe in core)
- Serde support behind feature flag
- Placeholder modules for future features (strategy, backtest, ndarray, simd)
