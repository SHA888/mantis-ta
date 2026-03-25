# mantis-ta

**Composable technical analysis and strategy engine for Rust.**

[![Crates.io](https://img.shields.io/crates/v/mantis-ta.svg)](https://crates.io/crates/mantis-ta)
[![Docs.rs](https://docs.rs/mantis-ta/badge.svg)](https://docs.rs/mantis-ta)
[![CI](https://github.com/SHA888/mantis-ta/actions/workflows/ci.yml/badge.svg)](https://github.com/SHA888/mantis-ta/actions)
[![Coverage](https://codecov.io/gh/SHA888/mantis-ta/branch/main/graph/badge.svg)](https://codecov.io/gh/SHA888/mantis-ta)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

---

Pure Rust technical indicators with a type-safe strategy composition API. No C dependencies. No FFI. No unsafe in the default build.

Every indicator is verified against [TA-Lib](https://ta-lib.org/) reference outputs.

```toml
[dependencies]
mantis-ta = "0.5.1"
```

## Quick Start

### Indicators — Streaming

Feed candles one at a time. Get values out. O(1) per update, zero heap allocations in the hot path.

```rust
use mantis_ta::prelude::*;

let mut ema = EMA::new(20);
let mut rsi = RSI::new(14);

for candle in candles.iter() {
    if let Some(ema_val) = ema.next(candle) {
        println!("EMA(20) = {:.2}", ema_val);
    }
    if let Some(rsi_val) = rsi.next(candle) {
        println!("RSI(14) = {:.2}", rsi_val);
    }
}
```

### Indicators — Batch

Compute over a full series at once. Returns `Vec<Option<f64>>` aligned with input candles (`None` during warmup).

```rust
use mantis_ta::prelude::*;

let sma_values = SMA::new(50).calculate(&candles);
let bb_values = BollingerBands::new(20, 2.0).calculate(&candles);

for (i, bb) in bb_values.iter().enumerate() {
    if let Some(bb) = bb {
        println!("Bar {}: Upper={:.2} Mid={:.2} Lower={:.2}", i, bb.upper, bb.middle, bb.lower);
    }
}
```

### Strategy Composition

Define complete trading strategies as composable, type-checked rules. Invalid strategies don't compile.

```rust
use mantis_ta::prelude::*;
use mantis_ta::strategy::*;

let strategy = Strategy::builder("Golden Cross Momentum")
    .timeframe(Timeframe::D1)
    .entry(
        all_of([
            ema(20).crosses_above(ema(50)),
            rsi(14).is_between(40.0, 65.0),
            volume().is_above(volume_sma(20).scaled(1.5)),
        ])
    )
    .exit(
        any_of([
            ema(20).crosses_below(ema(50)),
            rsi(14).is_above(80.0),
        ])
    )
    .stop_loss(StopLoss::atr_multiple(14, 2.0))
    .take_profit(TakeProfit::atr_multiple(14, 3.0))
    .max_position_size_pct(5.0)
    .build()?;

// Evaluate against historical data
let signals: Vec<Signal> = strategy.evaluate(&candles)?;

// Or stream live — same strategy, bar by bar
let mut engine = strategy.into_engine();
for candle in live_feed {
    match engine.next(&candle) {
        Signal::Entry(Side::Long)  => { /* open long */ },
        Signal::Exit(reason)       => { /* close position */ },
        Signal::Hold               => { /* wait */ },
        _ => {}
    }
}
```

### Backtesting

Honest simulation with realistic slippage, commissions, and next-bar execution.

```rust
use mantis_ta::backtest::*;

let result = backtest(&strategy, &candles, &BacktestConfig::default())?;

println!("Return:       {:.2}%", result.metrics.total_return_pct);
println!("Sharpe Ratio: {:.2}",  result.metrics.sharpe_ratio);
println!("Max Drawdown: {:.2}%", result.metrics.max_drawdown_pct);
println!("Win Rate:     {:.2}%", result.metrics.win_rate_pct);
println!("Trades:       {}",     result.metrics.total_trades);
```

## Available Indicators

### Trend
**v0.5.0 Batch A:** `SMA` · `EMA` · `WMA` · `DEMA` · `TEMA` · `MACD` · `ADX`
**Future:** `Ichimoku` · `Parabolic SAR` · `Supertrend`

### Momentum
**v0.5.0 Batch A:** `RSI` · `Stochastic` · `CCI` · `Williams %R` · `ROC`
**Future:** `MFI`

### Volatility
**v0.5.0 Batch A:** `Bollinger Bands` · `ATR` · `Standard Deviation`
**Future:** `Keltner Channels`

### Volume
`OBV` · `Volume SMA` · `VWAP` · `Accumulation/Distribution`

### Support/Resistance
`Pivot Points` · `Donchian Channels` · `Fibonacci Retracement`

See the [full indicator list](https://docs.rs/mantis-ta/latest/mantis_ta/indicators/) in the API docs.

## Features

```toml
[dependencies]
mantis-ta = { version = "0.5", features = ["strategy", "backtest"] }
```

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | ✓ | Serialize strategies, indicators, and results to JSON |
| `strategy` | ✓ | Strategy composition engine (v0.2.0+) |
| `backtest` | ✓ | Backtesting engine with metrics (v0.4.0+) |
| `ndarray` | | Interop with the `ndarray` ecosystem |
| `full-indicators` | | All 50+ indicators (default includes 30 most common) |
| `simd` | | SIMD-accelerated batch computation (uses `unsafe`) |
| `all` | | Everything |

## Design Principles

- **Correctness first.** Every indicator verified against TA-Lib (< 1e-10 relative error).
- **Streaming-first.** O(1) incremental updates for live data. Batch is also first-class.
- **Zero allocation in the hot path.** `next()` never heap-allocates.
- **No unsafe by default.** Safe Rust is fast enough.
- **Type system enforces validity.** A strategy without a stop-loss is a compile error, not a runtime surprise.
- **Honest backtesting.** No lookahead bias. Slippage and commissions are mandatory, not optional.

## Performance

Benchmarked on Apple M-series, single core:

| Operation | Time |
|-----------|------|
| EMA(20) per bar (streaming) | < 100 ns |
| RSI(14) batch, 2000 bars | < 15 µs |
| Strategy eval (5 conditions), 2000 bars | < 200 µs |
| Full backtest, 2 years daily | < 5 ms |

Run benchmarks yourself: `cargo bench`

## Custom Indicators

Implement the `Indicator` trait to create your own:

```rust
use mantis_ta::prelude::*;

pub struct MyIndicator {
    period: usize,
    buffer: Vec<f64>,
}

impl Indicator for MyIndicator {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.buffer.push(candle.close);
        if self.buffer.len() < self.period {
            return None;
        }
        // Your calculation here
        Some(self.buffer.iter().sum::<f64>() / self.period as f64)
    }

    fn warmup_period(&self) -> usize { self.period }
    fn reset(&mut self) { self.buffer.clear(); }
    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}
```

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](./CONTRIBUTING.md) before opening a PR.

Adding a new indicator? See the [Contributor Guide](./SPEC.md#10-contribution-guidelines) for the full checklist: implement the trait, add TA-Lib verification, write benchmarks, document it.

## License

Licensed under either of:

- [MIT License](./LICENSE-MIT)
- [Apache License, Version 2.0](./LICENSE-APACHE)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate shall be dual-licensed as above, without any additional terms or conditions.