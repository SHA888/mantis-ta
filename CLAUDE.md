# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**mantis-ta** is a composable technical analysis and strategy engine in pure Rust (no C/FFI dependencies). It provides streaming-first (O(1) per bar) indicators verified against TA-Lib to < 1e-10 relative error. Currently at v0.1.1 on crates.io.

## Common Commands

```bash
# Build
cargo build
cargo build --all-features

# Test
cargo test                    # All unit + integration tests
cargo test --all-features     # With all feature flags
cargo test --doc              # Doc examples only
cargo test <test_name>        # Single test by name

# Lint & Format
cargo fmt -- --check          # Check formatting
cargo fmt                     # Auto-format
cargo clippy -- -D warnings   # Lint (CI treats warnings as errors)

# Benchmarks (Criterion)
cargo bench                   # Run all benchmarks
cargo bench --bench indicators  # Indicator benchmarks only

# Generate TA-Lib reference data (requires Python + TA-Lib)
python fixtures/generate_references.py
```

## Architecture

### Core Traits (`src/indicators/mod.rs`)

All indicators implement the `Indicator` trait:
- `next(&mut self, candle: &Candle) -> Option<Self::Output>` — streaming, O(1)
- `calculate(&self, candles: &[Candle]) -> Vec<Option<Self::Output>>` — batch (default impl loops `next()`)
- `warmup_period(&self) -> usize` — returns `None` during warmup, then `Some(output)`
- `reset(&mut self)` / `clone_boxed(&self)` — for strategy engine reuse

`IncrementalIndicator` extends `Indicator` with `state()`/`restore()` for snapshotting.

### Module Layout

- `src/types/mod.rs` — `Candle`, `Signal`, `Side`, `MantisError`, output structs (`MacdOutput`, `BollingerOutput`, etc.)
- `src/indicators/{trend,momentum,volatility,volume,support_resistance}/` — indicators by category
- `src/utils/` — `RingBuf` (fixed-size circular buffer), math helpers, crossover detection
- `src/strategy/` and `src/backtest/` — placeholders for v0.2.0+

### Indicator Implementation Pattern

Every indicator follows this structure:
1. Struct with `period`, internal state, and `RingBuf` for windowed data
2. Private `update(value: f64) -> Option<Output>` with `#[inline]`
3. `Indicator` trait impl: `next()` extracts `candle.close` (or relevant fields) and calls `update()`
4. Constructor validates parameters with `assert!`
5. Composite indicators (e.g., MACD) own inner indicators and delegate

### Testing Strategy

- **Unit tests**: Inline `#[cfg(test)]` in each indicator module
- **Property tests**: `tests/property_tests.rs` — bounds checking, streaming/batch equivalence, fuzz with random candles
- **TA-Lib verification**: `tests/indicator_verification/` — compare against reference JSON in `fixtures/reference/`
- **Test helpers**: `tests/common/mod.rs` — `load_candles()`, `load_reference_series()` for fixture loading
- **Market data fixtures**: `fixtures/market_data/` — CSV files (AAPL, EURUSD, SPY)

### Feature Flags

- `serde` (default) — JSON serialization for all public types
- `ndarray`, `simd` — planned, currently no-op
- `full-indicators`, `strategy`, `backtest` — gating for future modules
- `all` — enables everything

## Conventions

- Zero heap allocation in `next()` hot path; use `RingBuf` for sliding windows
- All public items must have Rustdoc with runnable examples
- CI enforces: `cargo fmt --check`, `cargo clippy -- -D warnings`, all tests pass including doc tests
- Pre-commit hooks mirror CI checks (see `.pre-commit-config.yaml`)
- MSRV: Rust 1.75+
- Dual license: MIT OR Apache-2.0

## Adding a New Indicator

1. Create `src/indicators/{category}/{name}.rs`
2. Implement `Indicator` trait (use `RingBuf` for windows, `#[inline]` on `update`)
3. Re-export in `src/indicators/mod.rs` and `src/prelude.rs`
4. Add inline unit tests, TA-Lib verification test with reference JSON, and Criterion benchmark
5. Add Rustdoc example on the struct

## Key Reference Docs

- `SPEC.md` — detailed internal specification (API design, indicator checklist, architecture)
- `TODO.md` — version roadmap (v0.1.0 through v1.0.0)
