# mantis-ta — Internal Specification

> Internal planning document for maintainers and contributors.
> For the public overview, installation, and usage examples, see [README.md](./README.md).

---

## Table of Contents

1. [Vision & Positioning](#1-vision--positioning)
2. [Crate Architecture](#2-crate-architecture)
3. [Public API Design — Core Types](#3-public-api-design--core-types)
4. [Indicator Specification](#4-indicator-specification)
5. [Strategy Composition Engine — Internals](#5-strategy-composition-engine--internals)
6. [Backtesting Engine — Internals](#6-backtesting-engine--internals)
7. [Testing Strategy](#7-testing-strategy)
8. [Performance Targets](#8-performance-targets)
9. [Release Roadmap](#9-release-roadmap)
10. [Contribution Guidelines](#10-contribution-guidelines)
11. [License Rationale](#11-license-rationale)

---

## 1. Vision & Positioning

### 1.1 What mantis-ta Is

`mantis-ta` is the open-source computational core extracted from the MANTIS trading platform. It provides technical indicators, strategy composition, and backtesting as a standalone library — usable by anyone, not coupled to MANTIS.

### 1.2 What mantis-ta Is NOT

- **Not a trading platform.** No broker connections, no UI, no user accounts.
- **Not a data provider.** Operates on OHLCV data you provide.
- **Not a black-box ML system.** Rule-based and transparent.
- **Not a TA-Lib wrapper.** Every indicator is native Rust. No C dependencies. No FFI.

### 1.3 Design Principles

1. **Correctness over cleverness.** Every indicator verified against TA-Lib reference outputs.
2. **The type system is the first line of defense.** Invalid strategies should not compile.
3. **Streaming-first, batch-capable.** O(1) incremental updates AND batch computation — both first-class.
4. **Zero required dependencies beyond std.** Optional features enable `serde`, `ndarray`, etc.
5. **No unsafe in the default build.** Optional `simd` feature may use unsafe where justified.
6. **Honest backtesting.** No lookahead bias. Slippage and commission are mandatory.

### 1.4 Relationship to MANTIS Platform

```
Open Source (MIT + Apache 2.0):            Proprietary (MANTIS Platform):
┌────────────────────────────┐            ┌──────────────────────────────┐
│  mantis-ta crate           │            │  mantis-api (Axum server)    │
│  ├─ Technical indicators   │◄───────────│  mantis-broker (IBKR, MT5)   │
│  ├─ Strategy composition   │  depends   │  mantis-execution (orders)   │
│  ├─ Backtesting engine     │    on      │  mantis-ml (Layer 2 ML)      │
│  └─ Signal generation      │            │  Frontend (React dashboard)  │
└────────────────────────────┘            └──────────────────────────────┘
       ▲                                         ▲
       │ crates.io                               │ SaaS subscription
       │                                         │
   Community / OSS users                    Paying customers
```

The open-source crate builds credibility and community adoption. The commercial platform monetizes the non-technical user experience built on top. This is the open-core model (GitLab, Supabase, PostHog).

**Key discipline:** No MANTIS Platform-specific code in `mantis-ta`. If a feature only makes sense for the platform, it belongs in `mantis-engine` (the platform's internal crate), not here.

### 1.5 Target Users

- **Rust developers** building trading systems
- **Quant developers** migrating from Python (TA-Lib, backtrader)
- **Platform builders** who need a composable strategy engine as a foundation
- **Researchers** who want fast backtesting without Python overhead

---

## 2. Crate Architecture

### 2.1 Module Structure

```
mantis-ta/
├── Cargo.toml
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
├── SPEC.md                       # This document
├── CHANGELOG.md
├── CONTRIBUTING.md
│
├── src/
│   ├── lib.rs                    # Public API re-exports
│   ├── prelude.rs                # Convenience imports
│   │
│   ├── types/                    # Core data types
│   │   ├── mod.rs
│   │   ├── candle.rs             # OHLCV candle
│   │   ├── price.rs              # Price sources, fields
│   │   ├── signal.rs             # Entry/Exit/Hold signals
│   │   ├── side.rs               # Long/Short
│   │   └── timeframe.rs          # M1, M5, H1, D1, etc.
│   │
│   ├── indicators/               # Technical indicators
│   │   ├── mod.rs                # Indicator trait definition
│   │   ├── trait.rs              # Core traits
│   │   │
│   │   ├── trend/                # Trend-following
│   │   │   ├── mod.rs
│   │   │   ├── sma.rs
│   │   │   ├── ema.rs
│   │   │   ├── wma.rs
│   │   │   ├── dema.rs
│   │   │   ├── tema.rs
│   │   │   └── macd.rs
│   │   │
│   │   ├── momentum/             # Momentum oscillators
│   │   │   ├── mod.rs
│   │   │   ├── rsi.rs
│   │   │   ├── stochastic.rs
│   │   │   ├── cci.rs
│   │   │   ├── williams_r.rs
│   │   │   ├── roc.rs
│   │   │   └── mfi.rs
│   │   │
│   │   ├── volatility/           # Volatility measures
│   │   │   ├── mod.rs
│   │   │   ├── bollinger.rs
│   │   │   ├── atr.rs
│   │   │   ├── keltner.rs
│   │   │   └── std_dev.rs
│   │   │
│   │   ├── volume/               # Volume indicators
│   │   │   ├── mod.rs
│   │   │   ├── obv.rs
│   │   │   ├── volume_sma.rs
│   │   │   ├── vwap.rs
│   │   │   └── ad_line.rs
│   │   │
│   │   └── support_resistance/   # Support/Resistance
│   │       ├── mod.rs
│   │       ├── pivot_points.rs
│   │       └── donchian.rs
│   │
│   ├── strategy/                 # Strategy composition
│   │   ├── mod.rs
│   │   ├── builder.rs            # Fluent strategy builder
│   │   ├── condition.rs          # Condition types + logic
│   │   ├── operator.rs           # Comparison operators
│   │   ├── risk.rs               # Risk rule configuration
│   │   └── engine.rs             # Strategy evaluation engine
│   │
│   ├── backtest/                 # Backtesting engine
│   │   ├── mod.rs
│   │   ├── runner.rs             # Backtest execution loop
│   │   ├── config.rs             # Backtest configuration
│   │   ├── broker_sim.rs         # Simulated broker (fills, slippage)
│   │   ├── portfolio.rs          # Portfolio state tracking
│   │   └── metrics.rs            # Performance metrics calculation
│   │
│   └── utils/                    # Internal utilities
│       ├── mod.rs
│       ├── ringbuf.rs            # Fixed-size ring buffer for streaming
│       ├── math.rs               # Common math (rounding, etc.)
│       └── crossover.rs          # Cross-above/below detection
│
├── benches/                      # Criterion benchmarks
│   ├── indicators.rs
│   ├── strategy_eval.rs
│   └── backtest.rs
│
├── tests/                        # Integration tests
│   ├── indicator_verification/   # TA-Lib parity tests
│   │   ├── mod.rs
│   │   ├── test_sma.rs
│   │   ├── test_ema.rs
│   │   ├── test_rsi.rs
│   │   └── ...
│   ├── strategy_tests.rs
│   └── backtest_tests.rs
│
├── fixtures/                     # Test data
│   ├── generate_references.py    # TA-Lib reference generator
│   ├── reference/                # TA-Lib reference outputs
│   │   ├── sma_20_aapl.json
│   │   ├── ema_20_aapl.json
│   │   ├── rsi_14_aapl.json
│   │   └── ...
│   └── market_data/              # Sample OHLCV datasets
│       ├── aapl_daily_2y.csv
│       ├── eurusd_1h_1y.csv
│       └── spy_daily_5y.csv
│
└── examples/                     # Usage examples
    ├── basic_indicators.rs
    ├── streaming_ema.rs
    ├── golden_cross_strategy.rs
    ├── backtest_momentum.rs
    └── custom_indicator.rs
```

### 2.2 Feature Flags

```toml
[features]
default = ["serde"]
serde = ["dep:serde", "dep:serde_json"]
ndarray = ["dep:ndarray"]
simd = []
full-indicators = []
strategy = []
backtest = ["strategy"]
all = ["serde", "ndarray", "simd", "full-indicators", "strategy", "backtest"]
```

### 2.3 Cargo.toml

```toml
[package]
name = "mantis-ta"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
description = "Composable technical analysis and strategy engine for Rust"
documentation = "https://docs.rs/mantis-ta"
repository = "https://github.com/user/mantis-ta"
keywords = ["technical-analysis", "trading", "finance", "indicators", "backtesting"]
categories = ["algorithms", "mathematics", "finance"]

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
serde_json = { version = "1", optional = true }
ndarray = { version = "0.16", optional = true }
thiserror = "2"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
approx = "0.5"
csv = "1"
serde_json = "1"
rand = "0.8"

[[bench]]
name = "indicators"
harness = false

[[bench]]
name = "strategy_eval"
harness = false

[[bench]]
name = "backtest"
harness = false
```

---

## 3. Public API Design — Core Types

### 3.1 Data Types

```rust
/// A single OHLCV bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub timestamp: i64,     // Unix timestamp ms
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Which price field to use as input source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceSource {
    Open, High, Low, Close,
    HLC3,   // (H+L+C)/3
    OHLC4,  // (O+H+L+C)/4
    HL2,    // (H+L)/2
}

/// Trading direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Long, Short }

/// Signal emitted by strategy evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Entry(Side),
    Exit(ExitReason),
    Hold,
}

/// Why an exit was triggered.
#[derive(Debug, Clone, PartialEq)]
pub enum ExitReason {
    RuleTriggered,
    StopLoss,
    TakeProfit,
    TrailingStop,
    DailyLossLimit,
    DrawdownBreaker,
}

/// Candle timeframe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    M1, M5, M15, M30, H1, H4, D1, W1, MN1,
}
```

### 3.2 Indicator Traits

```rust
/// Core trait for all technical indicators.
pub trait Indicator: Send + Sync {
    type Output: Clone + std::fmt::Debug;

    /// Feed one candle, return current value. None during warmup.
    fn next(&mut self, candle: &Candle) -> Option<Self::Output>;

    /// Reset to initial state.
    fn reset(&mut self);

    /// Candles needed before first valid output.
    fn warmup_period(&self) -> usize;

    /// Batch compute over a candle series.
    fn calculate(&self, candles: &[Candle]) -> Vec<Option<Self::Output>> {
        let mut instance = self.clone_boxed();
        candles.iter().map(|c| instance.next(c)).collect()
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>>;
}

/// Indicator that supports O(1) incremental updates with checkpointing.
pub trait IncrementalIndicator: Indicator {
    type State: Clone;
    fn state(&self) -> Self::State;
    fn restore(&mut self, state: Self::State);
}
```

### 3.3 Multi-Output Indicator Types

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacdOutput {
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BollingerOutput {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StochasticOutput {
    pub k: f64,
    pub d: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PivotOutput {
    pub pp: f64,
    pub r1: f64, pub r2: f64, pub r3: f64,
    pub s1: f64, pub s2: f64, pub s3: f64,
}
```

### 3.4 Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MantisError {
    #[error("Invalid parameter: {param} = {value} ({reason})")]
    InvalidParameter {
        param: &'static str,
        value: String,
        reason: &'static str,
    },

    #[error("Insufficient data: need {required} candles, got {provided}")]
    InsufficientData { required: usize, provided: usize },

    #[error("Strategy validation failed: {0}")]
    StrategyValidation(String),

    #[error("Backtest error: {0}")]
    BacktestError(String),
}

pub type Result<T> = std::result::Result<T, MantisError>;
```

---

## 4. Indicator Specification

### 4.1 Indicator Roadmap

#### Tier 1 — v0.1.0 (MANTIS MVP, 10 indicators)

| # | Category | Indicator | Output | Streaming | Batch |
|---|----------|-----------|--------|-----------|-------|
| 1 | Trend | SMA | `f64` | ✓ | ✓ |
| 2 | Trend | EMA | `f64` | ✓ | ✓ |
| 3 | Trend | MACD | `MacdOutput` | ✓ | ✓ |
| 4 | Momentum | RSI | `f64` | ✓ | ✓ |
| 5 | Momentum | Stochastic Oscillator | `StochasticOutput` | ✓ | ✓ |
| 6 | Volatility | Bollinger Bands | `BollingerOutput` | ✓ | ✓ |
| 7 | Volatility | ATR | `f64` | ✓ | ✓ |
| 8 | Volume | Volume SMA | `f64` | ✓ | ✓ |
| 9 | Volume | OBV | `f64` | ✓ | ✓ |
| 10 | Support/Resistance | Pivot Points | `PivotOutput` | ✓ | ✓ |

#### Tier 2 — v0.2.0 (+15 indicators)

| # | Category | Indicator | Output |
|---|----------|-----------|--------|
| 11 | Trend | WMA | `f64` |
| 12 | Trend | DEMA | `f64` |
| 13 | Trend | TEMA | `f64` |
| 14 | Trend | Ichimoku Cloud | `IchimokuOutput` |
| 15 | Trend | Parabolic SAR | `f64` |
| 16 | Trend | ADX | `AdxOutput` |
| 17 | Momentum | CCI | `f64` |
| 18 | Momentum | Williams %R | `f64` |
| 19 | Momentum | ROC | `f64` |
| 20 | Momentum | MFI | `f64` |
| 21 | Volatility | Keltner Channels | `KeltnerOutput` |
| 22 | Volatility | Standard Deviation | `f64` |
| 23 | Volume | VWAP | `f64` |
| 24 | Volume | Accumulation/Distribution | `f64` |
| 25 | Support/Resistance | Donchian Channels | `DonchianOutput` |

#### Tier 3 — v0.3.0+ (+25 indicators)

| # | Category | Indicator |
|---|----------|-----------|
| 26 | Trend | VWMA |
| 27 | Trend | Hull Moving Average |
| 28 | Trend | ALMA |
| 29 | Trend | Supertrend |
| 30 | Trend | Aroon Oscillator |
| 31 | Momentum | TSI |
| 32 | Momentum | Ultimate Oscillator |
| 33 | Momentum | Awesome Oscillator |
| 34 | Momentum | Momentum (simple) |
| 35 | Momentum | TRIX |
| 36 | Volatility | Chaikin Volatility |
| 37 | Volatility | Historical Volatility |
| 38 | Volatility | Ulcer Index |
| 39 | Volume | Chaikin Money Flow |
| 40 | Volume | Force Index |
| 41 | Volume | Ease of Movement |
| 42 | Volume | Volume Profile (basic) |
| 43 | Candlestick | Doji detection |
| 44 | Candlestick | Engulfing pattern |
| 45 | Candlestick | Hammer / Hanging Man |
| 46 | Candlestick | Morning / Evening Star |
| 47 | Candlestick | Three White Soldiers / Black Crows |
| 48 | Support/Resistance | Fibonacci Retracement |
| 49 | Support/Resistance | Pivot Points (Fibonacci) |
| 50 | Support/Resistance | Pivot Points (Woodie) |

#### Tier 4 — Community-Driven (v0.4.0+)

Open to contributions. Any indicator with a clear mathematical definition, a reference implementation to test against, and a real use case.

### 4.2 Indicator Implementation Checklist

Every indicator MUST satisfy ALL of the following before merge:

1. **Correctness** — Output matches TA-Lib reference within `1e-10` relative error
2. **Streaming** — Implements `next(&mut self, candle: &Candle) -> Option<Output>`
3. **Batch** — Default `calculate()` works; override if a more efficient batch path exists
4. **Warmup** — `warmup_period()` returns exact candle count before first valid output
5. **Reset** — `reset()` restores to freshly-constructed state
6. **No panics** — All error states return `MantisError`
7. **No allocation in `next()`** — Streaming path uses pre-allocated ring buffers only
8. **Rustdoc** — Formula, parameters, defaults, common usage, runnable example
9. **Tests** — Unit tests + TA-Lib verification tests with fixture data
10. **Benchmarks** — Criterion benchmark for streaming (per-bar) and batch (2000 bars)

---

## 5. Strategy Composition Engine — Internals

### 5.1 Condition System Types

```rust
/// A single condition comparing an indicator output to a value.
pub struct Condition {
    left: IndicatorRef,
    operator: Operator,
    right: CompareTarget,
}

/// What to compare against.
pub enum CompareTarget {
    Value(f64),
    Indicator(IndicatorRef),
    Scaled(IndicatorRef, f64),
}

/// Comparison operators.
pub enum Operator {
    CrossesAbove,
    CrossesBelow,
    IsAbove,
    IsBelow,
    IsBetween(f64, f64),
    Equals,
    IsRising(u32),
    IsFalling(u32),
}

/// Group of conditions with AND/OR logic. One level of nesting.
pub enum ConditionGroup {
    AllOf(Vec<ConditionNode>),
    AnyOf(Vec<ConditionNode>),
}

pub enum ConditionNode {
    Condition(Condition),
    Group(ConditionGroup),
}
```

### 5.2 Convenience Functions

```rust
// Indicator reference constructors
pub fn sma(period: u32) -> IndicatorRef;
pub fn ema(period: u32) -> IndicatorRef;
pub fn rsi(period: u32) -> IndicatorRef;
pub fn macd(fast: u32, slow: u32, signal: u32) -> IndicatorRef;
pub fn macd_signal(fast: u32, slow: u32, signal: u32) -> IndicatorRef;
pub fn macd_histogram(fast: u32, slow: u32, signal: u32) -> IndicatorRef;
pub fn bollinger_upper(period: u32, std_dev: f64) -> IndicatorRef;
pub fn bollinger_lower(period: u32, std_dev: f64) -> IndicatorRef;
pub fn bollinger_middle(period: u32, std_dev: f64) -> IndicatorRef;
pub fn stochastic_k(k: u32, d: u32, slow: u32) -> IndicatorRef;
pub fn stochastic_d(k: u32, d: u32, slow: u32) -> IndicatorRef;
pub fn atr(period: u32) -> IndicatorRef;
pub fn volume() -> IndicatorRef;
pub fn volume_sma(period: u32) -> IndicatorRef;
pub fn obv() -> IndicatorRef;
pub fn price(field: PriceSource) -> IndicatorRef;
pub fn close() -> IndicatorRef;

// IndicatorRef methods for building conditions
impl IndicatorRef {
    pub fn crosses_above(self, other: impl Into<CompareTarget>) -> Condition;
    pub fn crosses_below(self, other: impl Into<CompareTarget>) -> Condition;
    pub fn is_above(self, other: impl Into<CompareTarget>) -> Condition;
    pub fn is_below(self, other: impl Into<CompareTarget>) -> Condition;
    pub fn is_between(self, low: f64, high: f64) -> Condition;
    pub fn is_rising(self, bars: u32) -> Condition;
    pub fn is_falling(self, bars: u32) -> Condition;
    pub fn scaled(self, factor: f64) -> ScaledIndicatorRef;
}

// Grouping
pub fn all_of(conditions: impl IntoIterator<Item = impl Into<ConditionNode>>) -> ConditionGroup;
pub fn any_of(conditions: impl IntoIterator<Item = impl Into<ConditionNode>>) -> ConditionGroup;
```

### 5.3 Strategy Validation Rules

`Strategy::build()` validates at construction time:

- At least one entry condition exists
- At least one exit condition exists
- Stop-loss is configured (mandatory)
- Take-profit is configured (mandatory)
- `max_position_size_pct` is between 0.1% and 100%
- `max_daily_loss_pct` is between 0.1% and 50%
- `max_drawdown_pct` is between 1% and 100%
- `max_concurrent_positions` ≥ 1
- All indicator periods ≥ 1
- Condition nesting does not exceed 2 levels
- Total conditions per group does not exceed 20
- No contradictory conditions (e.g., RSI > 70 AND RSI < 30 in same `all_of`)

### 5.4 Strategy Serialization

Strategies serialize to JSON (with `serde` feature) for storage and transport. This is critical for MANTIS Platform: the frontend sends strategies as JSON, the backend deserializes into typed `Strategy` structs.

```rust
let json = serde_json::to_string_pretty(&strategy)?;
let strategy: Strategy = serde_json::from_str(&json)?;
```

---

## 6. Backtesting Engine — Internals

### 6.1 Configuration

```rust
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub commission_per_trade: f64,
    pub commission_pct: f64,           // e.g., 0.001 = 0.1%
    pub slippage_pct: f64,             // e.g., 0.0005 = 0.05%
    pub execution: ExecutionModel,
    pub fractional_shares: bool,
    pub margin_requirement: f64,       // 1.0 = no leverage
}

pub enum ExecutionModel {
    NextBarOpen,      // Conservative (default)
    CurrentBarClose,  // Optimistic
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            commission_per_trade: 0.0,
            commission_pct: 0.001,
            slippage_pct: 0.0005,
            execution: ExecutionModel::NextBarOpen,
            fractional_shares: false,
            margin_requirement: 1.0,
        }
    }
}
```

### 6.2 Performance Metrics

```rust
pub struct BacktestMetrics {
    // Returns
    pub total_return_pct: f64,
    pub annualized_return_pct: f64,
    pub total_pnl: f64,

    // Risk-Adjusted
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,

    // Drawdown
    pub max_drawdown_pct: f64,
    pub max_drawdown_duration_bars: usize,
    pub recovery_factor: f64,

    // Trade Statistics
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub avg_trade_duration_bars: f64,
    pub largest_win: f64,
    pub largest_loss: f64,

    // Stress
    pub worst_single_bar: f64,
    pub longest_losing_streak: usize,
    pub max_consecutive_wins: usize,

    // Exposure
    pub avg_exposure_pct: f64,
    pub total_commission_paid: f64,
}
```

### 6.3 Integrity Rules

1. **No lookahead bias.** Indicators use only data up to the current bar.
2. **Next-bar execution by default.** Signals on bar N execute at bar N+1 open.
3. **Slippage modeling.** Every fill includes adverse slippage.
4. **Commission modeling.** Fixed-per-trade and percentage-of-value.
5. **No partial fills.** Full fill or nothing (partial fills are a future enhancement).
6. **Cash accounting.** Position sizing respects available cash.

### 6.4 Overfitting Warnings

Heuristic warnings emitted automatically:

- **Minimum trade count** — Warning if < 30 trades (insufficient sample size).
- **Parameter sensitivity** — Warning if ±1-2 period changes dramatically alter results.
- **Walk-forward validation** — Utility to split data into in-sample/out-of-sample windows.

---

## 7. Testing Strategy

### 7.1 Test Categories

| Category | Location | Purpose |
|----------|----------|---------|
| Unit | `src/` (inline) | Per-indicator: edge cases, NaN handling, warmup, reset |
| Verification | `tests/indicator_verification/` | TA-Lib parity with golden test data |
| Integration | `tests/` | Multi-component: builder → eval → signal accuracy |
| Property | `tests/` | Invariants: RSI ∈ [0,100], BB middle = SMA, streaming = batch |
| Fuzz | `tests/fuzz/` | Random candles never panic, extreme values handled |
| Benchmarks | `benches/` | Criterion: streaming per-bar, batch 2000 bars, strategy eval |

### 7.2 TA-Lib Verification Process

```
Step 1: Generate Reference Data (Python, one-time)
    fixtures/generate_references.py
    → Load AAPL daily data (2000+ bars)
    → Compute each indicator using TA-Lib
    → Export as JSON with full f64 precision

Step 2: Store as Fixtures
    fixtures/reference/sma_20_aapl.json
    {
      "indicator": "SMA",
      "params": {"period": 20},
      "source": "close",
      "dataset": "AAPL_daily",
      "talib_version": "0.4.28",
      "values": [null, null, ..., 150.2345678901234, ...]
    }

Step 3: Rust Tests Load + Compare
    #[test]
    fn sma_20_matches_talib() {
        let ref_data = load_reference("sma_20_aapl.json");
        let candles = load_candles("aapl_daily_2y.csv");
        let mut sma = SMA::new(20);

        for (i, candle) in candles.iter().enumerate() {
            let ours = sma.next(candle);
            match (ours, ref_data.values[i]) {
                (Some(a), Some(b)) => assert_relative_eq!(a, b, epsilon = 1e-10),
                (None, None) => {},
                _ => panic!("Warmup mismatch at bar {}", i),
            }
        }
    }

Step 4: CI Runs All Verification on Every Push
```

### 7.3 Reference Data Generator

```python
# fixtures/generate_references.py
"""
Generates TA-Lib reference outputs for verification.
Run once, commit JSON outputs. Re-run when adding indicators.

Requirements: pip install ta-lib pandas numpy
"""

import json, talib, pandas as pd, numpy as np

def generate_sma(df, periods=[5, 10, 20, 50, 100, 200]):
    for period in periods:
        values = talib.SMA(df['close'].values, timeperiod=period)
        save_reference('sma', {'period': period}, values)

def generate_ema(df, periods=[5, 10, 20, 50, 100, 200]):
    for period in periods:
        values = talib.EMA(df['close'].values, timeperiod=period)
        save_reference('ema', {'period': period}, values)

def generate_rsi(df, periods=[7, 14, 21]):
    for period in periods:
        values = talib.RSI(df['close'].values, timeperiod=period)
        save_reference('rsi', {'period': period}, values)

def generate_macd(df):
    macd, signal, hist = talib.MACD(
        df['close'].values, fastperiod=12, slowperiod=26, signalperiod=9
    )
    save_reference('macd', {'fast': 12, 'slow': 26, 'signal': 9}, {
        'macd_line': macd.tolist(),
        'signal_line': signal.tolist(),
        'histogram': hist.tolist(),
    })

# ... (similar for all indicators)

def save_reference(indicator, params, values):
    output = {
        'indicator': indicator,
        'params': params,
        'source': 'close',
        'dataset': 'AAPL_daily',
        'talib_version': talib.__version__,
        'values': [
            None if (isinstance(v, float) and np.isnan(v)) else v
            for v in (values if isinstance(values, list) else values.tolist())
        ]
    }
    param_str = '_'.join(f"{k}{v}" for k, v in sorted(params.items()))
    filename = f"reference/{indicator}_{param_str}_aapl.json"
    with open(filename, 'w') as f:
        json.dump(output, f, indent=2)

if __name__ == '__main__':
    df = pd.read_csv('market_data/aapl_daily_2y.csv')
    generate_sma(df)
    generate_ema(df)
    generate_rsi(df)
    generate_macd(df)
    print("Reference data generated.")
```

---

## 8. Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Single indicator `next()` | < 100 ns | Per-bar streaming |
| SMA(20) batch, 2000 bars | < 10 µs | |
| EMA(20) batch, 2000 bars | < 10 µs | |
| RSI(14) batch, 2000 bars | < 15 µs | |
| MACD batch, 2000 bars | < 25 µs | Three EMAs internally |
| Bollinger Bands batch, 2000 bars | < 20 µs | SMA + std dev |
| Strategy eval (5 conditions), 2000 bars | < 200 µs | All indicators included |
| Full backtest, 2yr daily, 1 instrument | < 5 ms | Including metrics |
| Full backtest, 2yr daily, 10 instruments | < 50 ms | Parallelizable |
| Memory per indicator instance | < 1 KB | Ring buffer + state |
| Heap allocations in `next()` | **Zero** | Mandatory |

---

## 9. Release Roadmap

### v0.1.0 — Foundation (Sprint 1-2 of MANTIS)

- 10 Tier 1 indicators — implemented, tested, verified
- Core types (Candle, Signal, PriceSource, etc.)
- Streaming + batch modes
- `serde` feature
- Criterion benchmarks
- Rustdoc with examples
- Published to crates.io

### v0.2.0 — Strategy Engine (Sprint 3-4)

- Strategy composition builder API
- Condition system with operators and grouping
- Strategy validation at build time
- Strategy serialization/deserialization
- Strategy evaluation (streaming + batch)
- +15 Tier 2 indicators (25 total)

### v0.3.0 — Backtesting (Sprint 5-6)

- Backtesting engine with simulated broker
- Portfolio tracking, cash accounting
- Full `BacktestMetrics`
- Slippage + commission modeling
- Overfitting warnings
- Walk-forward validation
- +25 Tier 3 indicators (50 total)

### v0.4.0 — Polish & Community (Sprint 7-8)

- Custom indicator trait for user extensions
- `ndarray` feature for scientific Rust interop
- Optional `simd` feature for batch acceleration
- Documentation site (mdBook)
- Contribution guidelines + good-first-issue labels
- Community contributions begin

### v1.0.0 — Stable (Post-MVP)

- API freeze — no breaking changes
- 50+ indicators, all TA-Lib verified
- Battle-tested via MANTIS Platform production
- Python bindings (separate crate: `mantis-ta-python`)
- WASM bindings (separate crate: `mantis-ta-wasm`)

---

## 10. Contribution Guidelines

### 10.1 Adding a New Indicator — Checklist

1. **Open an issue.** Describe indicator, link mathematical definition, link reference implementation.
2. **Claim it.** Comment so others don't duplicate.
3. **Implement in correct module.** `indicators/trend/`, `indicators/momentum/`, etc.
4. **Implement `Indicator` trait.** Both `next()` and optionally override `calculate()`.
5. **Generate TA-Lib reference.** Add to `generate_references.py`, commit JSON fixture.
6. **Write tests:** Unit (edge cases, NaN), verification (TA-Lib parity), property-based (invariants).
7. **Write Criterion benchmark.** Streaming + batch.
8. **Add Rustdoc.** Formula, params, defaults, runnable example.
9. **Open PR.** Link issue. CI must pass.

### 10.2 PR Requirements

- All existing tests pass
- New code has tests (unit + verification if applicable)
- `cargo clippy -- -D warnings` clean
- `cargo fmt` applied
- Rustdoc for all public items
- No new `unsafe` without explicit justification

### 10.3 Versioning

SemVer. Before v1.0.0, minor versions may include breaking changes. After v1.0.0, public API is stable.

### 10.4 Contributor License Agreement

Contributors sign a CLA before first PR merge. Grants maintainers the right to use contributions under both the open-source license and in commercial products built on `mantis-ta`. Does NOT transfer copyright.

---

## 11. License Rationale

Dual-licensed MIT + Apache 2.0.

**Why not just MIT?** Apache 2.0 includes an explicit patent grant from contributors, protecting users from patent claims.

**Why not AGPL?** Kills corporate adoption. The goal is maximum ecosystem adoption — the commercial moat is MANTIS Platform (SaaS, UX, broker integrations, ML), not the indicator math.

**Why dual?** Rust ecosystem convention (tokio, serde, axum). Users choose whichever fits.

**What stays proprietary:**

| Open Source (`mantis-ta`) | Proprietary (MANTIS Platform) |
|---------------------------|-------------------------------|
| Technical indicators | Web API server (Axum, auth, billing) |
| Strategy composition API | Broker adapters (IBKR, MT5) |
| Backtesting engine | ML Layer 2 (confidence, regime) |
| Signal generation | Risk/execution engine |
| | Frontend (React dashboard) |
| | Infrastructure (deployment, ops) |

---

*This specification is a living document. Last updated: February 2026.*