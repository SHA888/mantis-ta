# mantis-ta — TODO

Comprehensive implementation checklist derived from [SPEC.md](./SPEC.md) and [README.md](./README.md).
Items marked `[ADDITION]` are recommendations beyond the current SPEC.

---

## Version Roadmap

| Version | Theme | Platform Dependency |
|---------|-------|---------------------|
| **v0.1.0** | ✅ Tier 1 Indicators (10) + Core Types | `mantis-data` can compute indicators |
| **v0.1.1** | CI/CD + publish workflow + housekeeping | Dev velocity, automated quality gates |
| **v0.2.0** | Strategy Composition — Types + Builder | `mantis-core` can deserialize/validate strategies |
| **v0.3.0** | Strategy Evaluation — Batch + Streaming | `mantis-core` orchestrator can generate signals |
| **v0.4.0** | Backtesting Engine | Platform Phase 2: backtest UI can run strategies |
| **v0.5.0** | Tier 2 Indicators — Batch A (8) | Broader strategy options for users |
| **v0.6.0** | Tier 2 Indicators — Batch B (7) + ADX for Regime | `mantis-regime` MVP (ADX-based detection) |
| **v0.7.0** | Tier 3 Indicators — Batch A (13) + Candlestick Patterns | Advanced strategies, pattern recognition |
| **v0.8.0** | Tier 3 Indicators — Batch B (12) + Fibonacci/Pivot Variants | Full indicator catalog |
| **v0.9.0** | Polish — Custom Indicators, ndarray, SIMD, mdBook | Community readiness |
| **v1.0.0** | Stable — API Freeze, Bindings, Battle-Tested | Production-grade open-source release |

> **Rationale:** The platform (`mantis-core`, `mantis-execution`, `mantis-api`) is blocked on
> strategy composition (v0.2.0) and backtesting (v0.4.0). Indicator expansion (v0.5.0+) is
> valuable but not platform-blocking — the 10 Tier 1 indicators cover MVP strategy building.
> ADX is deliberately placed in v0.6.0 because `mantis-regime` MVP needs it for rule-based
> regime detection.

---

## ✅ v0.1.0 — Tier 1 Indicators + Core Types (PUBLISHED)

### 1. Project Scaffolding

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

### 2. Core Types

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

### 3. Indicator Infrastructure

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

### 4. Tier 1 Indicators (10)

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

### 5. Testing Infrastructure

- [x] `fixtures/generate_references.py` — TA-Lib reference data generator
- [x] Sample market data: `fixtures/market_data/aapl_daily_2y.csv`
- [x] Sample market data: `fixtures/market_data/eurusd_1h_1y.csv`
- [x] Sample market data: `fixtures/market_data/spy_daily_5y.csv`
- [x] TA-Lib reference JSONs for all Tier 1 indicators (SMA periods 5/10/20/50/100/200, EMA same, RSI 7/14/21, MACD 12/26/9, etc.)
- [x] Test harness: `load_reference()` and `load_candles()` helpers
- [x] Verification tests for each Tier 1 indicator (TA-Lib parity < 1e-10)
- [x] Unit tests per indicator: edge cases, NaN handling, warmup, reset
- [x] Property-based tests: RSI ∈ [0,100], BB middle = SMA, streaming output = batch output
- [x] Fuzz tests: random candles never panic, extreme values handled

### 6. Benchmarks

- [x] Criterion harness setup (`benches/indicators.rs`, `benches/strategy_eval.rs`, `benches/backtest.rs`)
- [x] Streaming per-bar benchmarks for each Tier 1 indicator (target: < 100 ns)
- [x] Batch 2000-bar benchmarks for each Tier 1 indicator (targets per SPEC §8)

### 7. Documentation

- [x] Rustdoc for all public types, traits, and functions
- [x] Runnable doc examples for each indicator
- [x] `examples/basic_indicators.rs`
- [x] `examples/streaming_ema.rs`
- [x] README badges: crates.io, docs.rs, CI, license
- [x] README quick-start examples

---

## v0.1.1 — CI/CD + Housekeeping (patch)

> Shipping CI/CD as a patch because it should have been in v0.1.0 and
> doesn't change public API. Unblocks automated quality gates for all
> subsequent development.

### CI/CD

- [x] GitHub Actions CI workflow (`.github/workflows/ci.yml`)
- [x] CI steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo test --all-features`
- [x] CI step: run TA-Lib verification tests (included in `cargo test`)
- [x] Crates.io publish workflow (manual `cargo publish` — no CI workflow needed)

### Housekeeping

- [x] Review and fix any open issues / bug reports from v0.1.0 (none reported)
- [x] Ensure `backtest` and `strategy` modules have clear "not yet implemented" docs (added module-level Rustdoc)
- [x] Verify docs.rs build renders correctly (all public items documented; backtest/strategy have clear future-feature docs)

---

## v0.2.0 — Strategy Composition: Types + Builder

> **Platform unblocks:** `mantis-core` can deserialize strategy JSON from the frontend
> Strategy Builder UI, validate it at build time, and store it. No evaluation yet —
> that's v0.3.0.

### Strategy Types

- [x] `Condition` struct (left indicator, operator, right compare target)
- [x] `CompareTarget` enum (Value, Indicator, Scaled)
- [x] `Operator` enum (CrossesAbove, CrossesBelow, IsAbove, IsBelow, IsBetween, Equals, IsRising, IsFalling)
- [x] `ConditionGroup` enum (AllOf, AnyOf)
- [x] `ConditionNode` enum (Condition, Group)
- [x] `StopLoss` type (ATR multiple, fixed %, trailing)
- [x] `TakeProfit` type (ATR multiple, fixed %)
- [x] `Strategy` struct

### Indicator References

- [x] `IndicatorRef` type + convenience constructors (`sma()`, `ema()`, `rsi()`, `macd()`, etc.)
- [x] `IndicatorRef` methods: `crosses_above`, `crosses_below`, `is_above`, `is_below`, `is_between`, `is_rising`, `is_falling`, `scaled`
- [x] `all_of()` and `any_of()` grouping functions

### Builder API

- [x] `Strategy::builder()` — fluent builder API
- [x] Builder methods: `timeframe`, `entry`, `exit`, `stop_loss`, `take_profit`, `max_position_size_pct`, `max_daily_loss_pct`, `max_drawdown_pct`, `max_concurrent_positions`
- [x] Strategy validation at `build()` time (all rules from SPEC §5.3)

### Serialization

- [x] Strategy serialization/deserialization (JSON, behind `serde` feature)
- [x] Ensure round-trip: `Strategy` → JSON → `Strategy` preserves all fields

### Testing

- [x] Unit tests for all `Operator` variants
- [x] Unit tests for `ConditionGroup` nesting (AllOf containing AnyOf, etc.)
- [x] Unit tests for builder validation (missing entry, missing stop-loss, too many conditions, etc.)
- [x] Round-trip serialization tests (serde derives in place)
- [x] Strategy composition tests (builder → struct integrity)

### Documentation

- [x] `examples/golden_cross_strategy.rs` — build a strategy, print its JSON
- [x] Rustdoc for all new public types (module-level and inline docs)

---

## v0.3.0 — Strategy Evaluation: Batch + Streaming

> **Platform unblocks:** `mantis-core` orchestrator can feed candles into a strategy
> and receive `Signal::Entry` / `Signal::Exit` / `Signal::Hold` back. This powers
> both paper trading and live signal generation.

### Evaluation Engine

- [x] Strategy evaluation — batch mode (`strategy.evaluate(&candles) -> Vec<Signal>`)
- [x] Strategy evaluation — streaming mode (`strategy.into_engine()` + `engine.next(&candle) -> Signal`)
- [x] Condition evaluator: resolve `IndicatorRef` to computed values, apply `Operator`
- [x] Cross-detection state management (previous bar values for CrossesAbove/Below)
- [x] Warmup handling: return `Signal::Hold` until all indicators have sufficient data

### Testing

- [x] Integration tests: builder → eval → signal accuracy (basic scenarios)
- [x] Golden Cross strategy: verify entry/exit signals against manual calculation
- [x] RSI Mean Reversion strategy: verify signals at known oversold/overbought points
- [x] Edge cases: strategy with single condition, maximum conditions, nested groups
- [x] Streaming vs. batch equivalence: same candles produce same signals in both modes

### Benchmarks

- [x] Strategy evaluation benchmark: 5 conditions, 2000 bars (target: < 200 µs)
- [x] CI step: run Criterion benchmarks (report only, no gate)

### Documentation

- [x] Update `examples/golden_cross_strategy.rs` — now includes evaluation + signal output

---

## v0.4.0 — Backtesting Engine

> **Platform unblocks:** Platform Phase 2 (Validation Engine). The backtest UI can
> submit a strategy + historical data and receive metrics, equity curve, and trade log.

### Core Engine

- [x] `BacktestConfig` struct (initial capital, commission, slippage, execution model, fractional shares, margin)
- [x] `ExecutionModel` enum (NextBarOpen, CurrentBarClose)
- [x] `BacktestConfig::default()` implementation
- [x] `backtest()` runner function — main execution loop
- [x] `BrokerSim` — simulated broker (fills with slippage)
- [x] `Portfolio` — portfolio state tracking, cash accounting, position sizing

### Metrics

- [x] `BacktestMetrics` struct (all fields from SPEC §6.2: returns, risk-adjusted, drawdown, trade stats, stress, exposure)
- [x] Metrics calculation from trade history
- [x] Trade log output: entry/exit timestamps, prices, P&L, exit reason, holding period

### Integrity Rules

- [x] No lookahead bias: indicators see only data up to current bar
- [x] Next-bar execution: entries/exits fill at next bar's open (default)
- [x] Slippage modeling (configurable, default 0.1% equities, 0.05% forex)
- [x] Commission modeling (flat fee or percentage)
- [x] Cash accounting: cannot buy more than available cash (no hidden margin)
- [x] No partial fills in MVP

### Overfitting Safeguards

- [x] Minimum trade count warning (< 30 trades = statistically unreliable)
- [x] Excessive condition warning (> 6–7 conditions = potential overfitting)
- [x] Parameter sensitivity report (±10% parameter variation impact)
- [x] Walk-forward validation utility (in-sample + out-of-sample split, report both)

### Testing

- [x] Backtest integration tests: known strategy on known data → verify metrics match hand calculation
- [x] No-lookahead-bias test: strategy with future-dependent condition must not generate signals
- [x] Cash accounting test: cannot open position larger than available capital
- [x] Commission/slippage test: verify they reduce returns appropriately
- [x] Edge cases: strategy that never trades, strategy that's always in position

### Benchmarks

- [x] Full backtest benchmark: 2yr daily, 1 instrument (target: < 5 ms)
- [x] Full backtest benchmark: 2yr daily, 10 instruments (target: < 50 ms)

### Documentation

- [x] `examples/backtest_momentum.rs` — full backtest with metrics output
- [x] Rustdoc for all backtest public types

---

## v0.5.0 — Tier 2 Indicators: Batch A (8 indicators)

> **Released as v0.5.0** (v0.4.0 Backtesting Engine was completed but released separately).
> Prioritized by usefulness for common strategy patterns. These indicators
> expand what users can build in the Strategy Builder without needing to
> wait for the full Tier 2 set.

- [x] ADX (Average Directional Index) — `AdxOutput` ← **priority: needed by `mantis-regime` MVP**
- [x] WMA (Weighted Moving Average) — `f64`
- [x] DEMA (Double Exponential Moving Average) — `f64`
- [x] TEMA (Triple Exponential Moving Average) — `f64`
- [x] CCI (Commodity Channel Index) — `f64`
- [x] Williams %R — `f64`
- [x] ROC (Rate of Change) — `f64`
- [x] Standard Deviation — `f64`

### Testing & Benchmarks

- [x] TA-Lib reference JSONs for all Batch A indicators
- [x] Verification tests (TA-Lib parity < 1e-10)
- [x] Unit tests per indicator (edge cases, warmup, reset)
- [x] Streaming + batch benchmarks

### Strategy Integration

- [x] Add `IndicatorRef` convenience constructors for new indicators (`adx()`, `wma()`, `cci()`, etc.)
- [x] Verify new indicators work in strategy builder → eval → signal flow

---

## v0.5.3 — Rust Upgrade

- [x] Set `edition = "2024"` in all workspace/crate Cargo.toml files
- [x] Set `rust-version = "1.85"` (MSRV) in root Cargo.toml
- [x] Run `cargo fix --edition` to auto-migrate code
- [x] Resolve remaining compiler warnings/errors
  - `unsafe extern` blocks
  - `#[unsafe(no_mangle)]` attributes
  - `unsafe {}` blocks inside `unsafe fn`
  - `r#gen` if `gen` used as identifier
- [x] Run `cargo clippy` and `cargo test` on clean build
- [x] Update CI toolchain pin (e.g., `rust-toolchain.toml`) to >= 1.85
- [x] Update docs and README references to edition/MSRV

---

## v0.6.0 — Tier 2 Indicators: Batch B (7 indicators)

> Completes Tier 2. ADX from v0.5.0 now enables `mantis-regime` to
> implement rule-based regime detection (ADX > 25 = trending, etc.).

- [ ] Ichimoku Cloud — `IchimokuOutput`
- [ ] Parabolic SAR — `f64`
- [ ] MFI (Money Flow Index) — `f64`
- [ ] Keltner Channels — `KeltnerOutput`
- [ ] VWAP (Volume Weighted Average Price) — `f64`
- [ ] Accumulation/Distribution Line — `f64`
- [ ] Donchian Channels — `DonchianOutput`

### Testing & Benchmarks

- [ ] TA-Lib reference JSONs for all Batch B indicators
- [ ] Verification tests (TA-Lib parity < 1e-10)
- [ ] Unit tests per indicator (edge cases, warmup, reset)
- [ ] Streaming + batch benchmarks

### Strategy Integration

- [ ] Add `IndicatorRef` convenience constructors for new indicators
- [ ] Verify new indicators work in strategy builder → eval → signal flow

---

## v0.7.0 — Tier 3 Indicators: Batch A (13 indicators)

> Advanced moving averages, oscillators, and volatility measures.

### Advanced Moving Averages

- [ ] VWMA (Volume Weighted Moving Average)
- [ ] Hull Moving Average
- [ ] ALMA (Arnaud Legoux Moving Average)

### Advanced Trend/Momentum

- [ ] Supertrend
- [ ] Aroon Oscillator
- [ ] TSI (True Strength Index)
- [ ] Ultimate Oscillator
- [ ] Awesome Oscillator
- [ ] Momentum (simple)
- [ ] TRIX

### Advanced Volatility/Volume

- [ ] Chaikin Volatility
- [ ] Historical Volatility
- [ ] Ulcer Index

### Testing & Benchmarks

- [ ] TA-Lib reference data (where available; some indicators not in TA-Lib — use alternative verified references)
- [ ] Verification tests, unit tests, streaming + batch benchmarks

### Strategy Integration

- [ ] Add `IndicatorRef` convenience constructors for new indicators

---

## v0.8.0 — Tier 3 Indicators: Batch B (12) + Candlestick Patterns

> Volume-based indicators, candlestick pattern detection, and pivot variants.

### Volume-Based

- [ ] Chaikin Money Flow
- [ ] Force Index
- [ ] Ease of Movement
- [ ] Volume Profile (basic)

### Candlestick Patterns

- [ ] Doji detection
- [ ] Engulfing pattern
- [ ] Hammer / Hanging Man
- [ ] Morning / Evening Star
- [ ] Three White Soldiers / Black Crows

### Support/Resistance Variants

- [ ] Fibonacci Retracement
- [ ] Pivot Points (Fibonacci variant)
- [ ] Pivot Points (Woodie variant)

### Testing & Benchmarks

- [ ] Pattern detection tests against known chart formations
- [ ] Unit tests, streaming + batch benchmarks

### Strategy Integration

- [ ] Add candlestick pattern support to `ConditionNode` (pattern detected as boolean condition)
- [ ] Add `IndicatorRef` convenience constructors for new indicators

---

## v0.9.0 — Polish & Community Readiness

### Custom Indicator Support

- [ ] Document the custom indicator pattern (public `Indicator` trait is sufficient — write the guide)
- [ ] `examples/custom_indicator.rs`

### Optional Features

- [ ] `ndarray` feature — interop with ndarray ecosystem
- [ ] `simd` feature — SIMD-accelerated batch computation (uses `unsafe`)
- [ ] `[ADDITION]` Security audit for `simd` feature `unsafe` code

### Documentation Site

- [ ] mdBook documentation site
- [ ] `[ADDITION]` mdBook content plan — pages: Getting Started, Indicator Catalog, Strategy Guide, Backtest Guide, Custom Indicators, API Reference

### Community

- [ ] Contribution guidelines finalized + good-first-issue labels on GitHub
- [ ] `[ADDITION]` Formal Tier 4 indicator acceptance process — GitHub issue template, review criteria checklist, required TA-Lib reference
- [ ] Tier 4 community-driven indicator acceptance (open process)

### CI Enhancements

- [ ] `[ADDITION]` Performance regression CI — automated benchmark comparison on PRs (e.g., `critcmp` or GitHub Action for Criterion)

---

## v1.0.0 — Stable Release

> API freeze. No breaking changes after this release.

### Stability

- [ ] `[ADDITION]` API stability review checklist — enumerate public surface, review naming conventions, ensure no accidental exposures
- [ ] `[ADDITION]` Migration/deprecation guide for pre-1.0 breaking changes
- [ ] API freeze — no breaking changes after this release
- [ ] 50+ indicators, all verified against reference implementations

### Bindings

- [ ] `[ADDITION]` Python/WASM binding scope definition — which APIs to expose, packaging strategy (PyPI / npm), CI for bindings
- [ ] Python bindings (separate crate: `mantis-ta-python`)
- [ ] WASM bindings (separate crate: `mantis-ta-wasm`)

### Production Validation

- [ ] Battle-tested via MANTIS Platform production usage

---

## Cross-Cutting Concerns (ongoing, every release)

### Quality Gates (every PR)

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test` + `cargo test --all-features`
- TA-Lib verification tests pass
- New public types have Rustdoc + examples
- New indicators have Criterion benchmarks

### Changelog

- Update `CHANGELOG.md` with every version (following Keep a Changelog format)

### Versioning Policy

- **Patch** (0.x.Y): Bug fixes, CI/CD, docs — no public API changes
- **Minor** (0.X.0): New features, new indicators, new public types — may include breaking changes pre-1.0
- **Major** (X.0.0): Reserved for post-1.0 breaking changes

---

## Platform Dependency Map

```
mantis-ta version    Platform component unblocked
─────────────────    ──────────────────────────────────
v0.1.0 (done)        mantis-data: indicator computation
v0.2.0               mantis-core: strategy deserialization + validation
                     Frontend: Strategy Builder can save valid strategies
v0.3.0               mantis-core: orchestrator signal generation
                     Platform: paper trading signal loop
v0.4.0               Platform Phase 2: backtest UI + metrics display
v0.5.0               mantis-regime: ADX-based regime detection MVP
v0.6.0+              Expanded strategy options for users (non-blocking)
v1.0.0               Production-grade open-source release
```
