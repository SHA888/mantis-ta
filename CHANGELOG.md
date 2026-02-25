# Changelog

All notable changes to this project will be documented in this file.

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
