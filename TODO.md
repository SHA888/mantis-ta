# mantis-ta — TODO

Comprehensive implementation checklist derived from [SPEC.md](./SPEC.md) and [README.md](./README.md).
Items marked `[ADDITION]` are recommendations beyond the current SPEC.

---

## 1. Project Scaffolding

- [x] Create `Cargo.toml` with package metadata, dependencies, dev-dependencies, feature flags, and bench targets
- [x] Create `LICENSE-MIT`
- [x] Create `LICENSE-APACHE`
- [x] Create `CHANGELOG.md`
- [x] Create `CONTRIBUTING.md`
- [x] Create `src/lib.rs` — public API re-exports
- [x] Create `src/prelude.rs` — convenience imports
- [x] Create module directory structure:
  - [x] `src/types/`
  - [x] `src/indicators/` (with `trend/`, `momentum/`, `volatility/`, `volume/`, `support_resistance/` sub-modules)
  - [x] `src/strategy/`
  - [x] `src/backtest/`
  - [x] `src/utils/`
- [x] Create `benches/` directory with Criterion harness stubs
- [x] Create `tests/` directory structure (`indicator_verification/`, etc.)
- [x] Create `fixtures/` directory (`reference/`, `market_data/`)
- [x] Create `examples/` directory

## 2. Core Types

- [x] `Candle` struct (timestamp, OHLCV)
- [x] `PriceSource` enum (Open, High, Low, Close, HLC3, OHLC4, HL2)
- [x] `Side` enum (Long, Short)
- [x] `Signal` enum (Entry, Exit, Hold)
- [x] `ExitReason` enum (RuleTriggered, StopLoss, TakeProfit, TrailingStop, DailyLossLimit, DrawdownBreaker)
- [x] `Timeframe` enum (M1, M5, M15, M30, H1, H4, D1, W1, MN1)
- [x] `MantisError` enum with `thiserror` (InvalidParameter, InsufficientData, StrategyValidation, BacktestError)
- [x] `Result<T>` type alias
- [x] `MacdOutput` struct
- [x] `BollingerOutput` struct
- [x] `StochasticOutput` struct
- [x] `PivotOutput` struct
- [x] Implement `serde` derives behind feature flag for all public types

## 3. Indicator Infrastructure

- [x] `Indicator` trait (`next`, `reset`, `warmup_period`, `calculate`, `clone_boxed`)
- [x] `IncrementalIndicator` trait (`state`, `restore`)
- [x] `src/utils/ringbuf.rs` — fixed-size ring buffer for streaming indicators
- [x] `src/utils/math.rs` — common math utilities (rounding, etc.)
- [x] `src/utils/crossover.rs` — cross-above/below detection helpers
- [x] Wire up `prelude.rs` with all public indicator re-exports

> **Indicator implementation checklist** (applies to every indicator below):
> Each indicator must satisfy all 10 points from SPEC §4.2 before merge:
> correctness (TA-Lib < 1e-10), streaming `next()`, batch `calculate()`, `warmup_period()`,
> `reset()`, no panics, zero allocation in `next()`, Rustdoc, tests (unit + TA-Lib verification),
> and Criterion benchmarks (streaming + batch).

## 4. Tier 1 Indicators — v0.1.0 (10 indicators)

- [x] SMA (Simple Moving Average) — `f64`
- [x] EMA (Exponential Moving Average) — `f64`
- [x] MACD (Moving Average Convergence Divergence) — `MacdOutput`
- [x] RSI (Relative Strength Index) — `f64`
- [x] Stochastic Oscillator — `StochasticOutput`
- [x] Bollinger Bands — `BollingerOutput`
- [x] ATR (Average True Range) — `f64`
- [x] Volume SMA — `f64`
- [x] OBV (On-Balance Volume) — `f64`
- [x] Pivot Points — `PivotOutput`

## 5. Tier 2 Indicators — v0.2.0 (+15 indicators)

- [ ] WMA (Weighted Moving Average) — `f64`
- [ ] DEMA (Double Exponential Moving Average) — `f64`
- [ ] TEMA (Triple Exponential Moving Average) — `f64`
- [ ] Ichimoku Cloud — `IchimokuOutput`
- [ ] Parabolic SAR — `f64`
- [ ] ADX (Average Directional Index) — `AdxOutput`
- [ ] CCI (Commodity Channel Index) — `f64`
- [ ] Williams %R — `f64`
- [ ] ROC (Rate of Change) — `f64`
- [ ] MFI (Money Flow Index) — `f64`
- [ ] Keltner Channels — `KeltnerOutput`
- [ ] Standard Deviation — `f64`
- [ ] VWAP (Volume Weighted Average Price) — `f64`
- [ ] Accumulation/Distribution Line — `f64`
- [ ] Donchian Channels — `DonchianOutput`

## 6. Tier 3 Indicators — v0.3.0+ (+25 indicators)

- [ ] VWMA (Volume Weighted Moving Average)
- [ ] Hull Moving Average
- [ ] ALMA (Arnaud Legoux Moving Average)
- [ ] Supertrend
- [ ] Aroon Oscillator
- [ ] TSI (True Strength Index)
- [ ] Ultimate Oscillator
- [ ] Awesome Oscillator
- [ ] Momentum (simple)
- [ ] TRIX
- [ ] Chaikin Volatility
- [ ] Historical Volatility
- [ ] Ulcer Index
- [ ] Chaikin Money Flow
- [ ] Force Index
- [ ] Ease of Movement
- [ ] Volume Profile (basic)
- [ ] Doji detection
- [ ] Engulfing pattern
- [ ] Hammer / Hanging Man
- [ ] Morning / Evening Star
- [ ] Three White Soldiers / Black Crows
- [ ] Fibonacci Retracement
- [ ] Pivot Points (Fibonacci variant)
- [ ] Pivot Points (Woodie variant)

## 7. Strategy Composition Engine — v0.2.0

- [ ] `Condition` struct (left indicator, operator, right compare target)
- [ ] `CompareTarget` enum (Value, Indicator, Scaled)
- [ ] `Operator` enum (CrossesAbove, CrossesBelow, IsAbove, IsBelow, IsBetween, Equals, IsRising, IsFalling)
- [ ] `ConditionGroup` enum (AllOf, AnyOf)
- [ ] `ConditionNode` enum (Condition, Group)
- [ ] `IndicatorRef` type + convenience constructors (`sma()`, `ema()`, `rsi()`, `macd()`, etc.)
- [ ] `IndicatorRef` methods: `crosses_above`, `crosses_below`, `is_above`, `is_below`, `is_between`, `is_rising`, `is_falling`, `scaled`
- [ ] `all_of()` and `any_of()` grouping functions
- [ ] `Strategy` struct
- [ ] `Strategy::builder()` — fluent builder API
- [ ] Builder methods: `timeframe`, `entry`, `exit`, `stop_loss`, `take_profit`, `max_position_size_pct`, `max_daily_loss_pct`, `max_drawdown_pct`, `max_concurrent_positions`
- [ ] `StopLoss` type (ATR multiple, fixed %, etc.)
- [ ] `TakeProfit` type (ATR multiple, fixed %, etc.)
- [ ] Strategy validation at `build()` time (all rules from SPEC §5.3)
- [ ] Strategy evaluation — batch mode (`strategy.evaluate(&candles)`)
- [ ] Strategy evaluation — streaming mode (`strategy.into_engine()` + `engine.next(&candle)`)
- [ ] Strategy serialization/deserialization (JSON, behind `serde` feature)

## 8. Backtesting Engine — v0.3.0

- [ ] `BacktestConfig` struct (initial capital, commission, slippage, execution model, fractional shares, margin)
- [ ] `ExecutionModel` enum (NextBarOpen, CurrentBarClose)
- [ ] `BacktestConfig::default()` implementation
- [ ] `backtest()` runner function — main execution loop
- [ ] `BrokerSim` — simulated broker (fills with slippage)
- [ ] `Portfolio` — portfolio state tracking, cash accounting, position sizing
- [ ] `BacktestMetrics` struct (all fields from SPEC §6.2: returns, risk-adjusted, drawdown, trade stats, stress, exposure)
- [ ] Metrics calculation from trade history
- [ ] Integrity rules enforcement: no lookahead bias, next-bar execution, slippage, commission, no partial fills, cash accounting
- [ ] Overfitting warnings: minimum trade count (< 30), parameter sensitivity, walk-forward validation utility

## 9. Testing Infrastructure

- [x] `fixtures/generate_references.py` — TA-Lib reference data generator — v0.1.0
- [x] Sample market data: `fixtures/market_data/aapl_daily_2y.csv` — v0.1.0
- [x] Sample market data: `fixtures/market_data/eurusd_1h_1y.csv` — v0.1.0
- [x] Sample market data: `fixtures/market_data/spy_daily_5y.csv` — v0.1.0
- [x] TA-Lib reference JSONs for all Tier 1 indicators (SMA periods 5/10/20/50/100/200, EMA same, RSI 7/14/21, MACD 12/26/9, etc.) — v0.1.0
- [x] Test harness: `load_reference()` and `load_candles()` helpers — v0.1.0
- [ ] Verification tests for each Tier 1 indicator (TA-Lib parity < 1e-10) — v0.1.0
- [ ] Unit tests per indicator: edge cases, NaN handling, warmup, reset — v0.1.0
- [ ] Property-based tests: RSI ∈ [0,100], BB middle = SMA, streaming output = batch output — v0.1.0
- [ ] Fuzz tests: random candles never panic, extreme values handled — v0.1.0
- [ ] Integration tests: builder → eval → signal accuracy — v0.2.0
- [ ] Strategy composition tests — v0.2.0
- [ ] Backtest integration tests — v0.3.0

## 10. Benchmarks

- [ ] Criterion harness setup (`benches/indicators.rs`, `benches/strategy_eval.rs`, `benches/backtest.rs`) — v0.1.0
- [ ] Streaming per-bar benchmarks for each Tier 1 indicator (target: < 100 ns) — v0.1.0
- [ ] Batch 2000-bar benchmarks for each Tier 1 indicator (targets per SPEC §8) — v0.1.0
- [ ] Strategy evaluation benchmark: 5 conditions, 2000 bars (target: < 200 µs) — v0.2.0
- [ ] Full backtest benchmark: 2yr daily, 1 instrument (target: < 5 ms) — v0.3.0
- [ ] Full backtest benchmark: 2yr daily, 10 instruments (target: < 50 ms) — v0.3.0

## 11. Documentation

- [ ] Rustdoc for all public types, traits, and functions — v0.1.0
- [ ] Runnable doc examples for each indicator — v0.1.0
- [ ] `examples/basic_indicators.rs` — v0.1.0
- [ ] `examples/streaming_ema.rs` — v0.1.0
- [ ] `examples/golden_cross_strategy.rs` — v0.2.0
- [ ] `examples/backtest_momentum.rs` — v0.3.0
- [ ] `examples/custom_indicator.rs` — v0.4.0
- [ ] README badges: crates.io, docs.rs, CI, license — v0.1.0
- [ ] README quick-start examples (already drafted)

## 12. v0.4.0 — Polish & Community

- [ ] Custom indicator trait for user extensions (public `Indicator` trait is sufficient, but document the pattern)
- [ ] `ndarray` feature — interop with ndarray ecosystem
- [ ] `simd` feature — SIMD-accelerated batch computation (uses `unsafe`)
- [ ] mdBook documentation site
- [ ] Contribution guidelines finalized + good-first-issue labels on GitHub
- [ ] Tier 4 community-driven indicator acceptance (open process)

## 13. v1.0.0 — Stable

- [ ] API freeze — no breaking changes after this release
- [ ] 50+ indicators, all TA-Lib verified
- [ ] Battle-tested via MANTIS Platform production usage
- [ ] Python bindings (separate crate: `mantis-ta-python`)
- [ ] WASM bindings (separate crate: `mantis-ta-wasm`)

## 14. CI/CD

- [ ] GitHub Actions CI workflow (`.github/workflows/ci.yml`) — v0.1.0
- [ ] CI steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo test --all-features` — v0.1.0
- [ ] CI step: run TA-Lib verification tests — v0.1.0
- [ ] CI step: run Criterion benchmarks (report only, no gate) — v0.2.0
- [ ] Crates.io publish workflow (manual trigger or tag-based) — v0.1.0

## 15. Beyond SPEC

> Items below are **not in the current SPEC** but are recommended additions
> to close gaps between v0.4.0 and v1.0.0.

- [ ] `[ADDITION]` Migration/deprecation guide for pre-1.0 breaking changes (document policy for minor version bumps)
- [ ] `[ADDITION]` mdBook content plan — pages: Getting Started, Indicator Catalog, Strategy Guide, Backtest Guide, Custom Indicators, API Reference
- [ ] `[ADDITION]` Python/WASM binding scope definition — which APIs to expose, packaging strategy (PyPI / npm), CI for bindings
- [ ] `[ADDITION]` Performance regression CI — automated benchmark comparison on PRs (e.g., `critcmp` or GitHub Action for Criterion)
- [ ] `[ADDITION]` Security audit for `simd` feature `unsafe` code before 1.0 release
- [ ] `[ADDITION]` Formal Tier 4 indicator acceptance process — GitHub issue template, review criteria checklist, required TA-Lib reference
- [ ] `[ADDITION]` API stability review checklist before 1.0 freeze — enumerate public surface, review naming conventions, ensure no accidental exposures
