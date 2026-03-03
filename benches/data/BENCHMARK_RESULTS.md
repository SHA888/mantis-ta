# Backtest Engine Performance Benchmarks

## Summary

The backtest engine has been benchmarked on 2-year daily OHLC data (SPY) with a simple momentum strategy (SMA(20) crossover).

### Results

| Benchmark | Mean Time | Target | Status |
|-----------|-----------|--------|--------|
| 1 instrument (2yr daily) | **1.46 ms** | < 5 ms | ✅ Pass |
| 10 instruments (2yr daily) | **14.33 ms** | < 50 ms | ✅ Pass |

### Performance Characteristics

- **Single backtest throughput**: ~686 backtests/second
- **10-instrument throughput**: ~70 instrument sets/second
- **Scaling factor**: ~9.8x for 10x instruments (near-linear)

### Benchmark Setup

**Strategy**: Simple momentum
- Entry: SMA(20) < 100.0
- Exit: SMA(20) > 100.0
- Stop Loss: 2% fixed

**Data**: SPY daily OHLC, ~2 years (~500 bars)

**Config**: Default BacktestConfig
- Initial capital: $100,000
- Commission: 0.001 (0.1%)
- Slippage: 0.001 (0.1%)

### Detailed Results

#### 1 Instrument (2yr daily)
```
Mean:       1.4590 ms
Median:     1.4328 ms
Std Dev:    70.582 µs
Min:        1.3986 ms
Max:        1.4815 ms
```

#### 10 Instruments (2yr daily)
```
Mean:       14.328 ms
Median:     14.126 ms
Std Dev:    768.89 µs
Min:        13.863 ms
Max:        14.548 ms
```

### Running Benchmarks

To run the benchmarks locally:

```bash
cargo bench --bench backtest --features backtest
```

For detailed output with verbose statistics:

```bash
cargo bench --bench backtest --features backtest -- --verbose
```

### Data Files

- `backtest_1_instrument_estimates.json` - Statistical estimates for 1-instrument benchmark
- `backtest_10_instruments_estimates.json` - Statistical estimates for 10-instrument benchmark

These files contain detailed statistical analysis including confidence intervals, outlier detection, and regression analysis.
