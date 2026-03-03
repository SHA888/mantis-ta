use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mantis_ta::backtest::{backtest, BacktestConfig};
use mantis_ta::strategy::indicator_ref::IndicatorRef;
use mantis_ta::strategy::{Strategy, StopLoss};

mod common;
use common::load_candles;

fn bench_backtest_1_instrument(c: &mut Criterion) {
    let candles = load_candles("market_data/spy_daily_5y.csv");
    
    // Simple momentum strategy: buy when SMA(20) < 100, sell when SMA(20) > 100
    let strategy = Strategy::builder("momentum_1yr")
        .entry(IndicatorRef::sma(20).is_below(100.0))
        .exit(IndicatorRef::sma(20).is_above(100.0))
        .stop_loss(StopLoss::FixedPercent(2.0))
        .build()
        .unwrap();
    
    let config = BacktestConfig::default();
    
    c.bench_function("backtest_2yr_daily_1_instrument", |b| {
        b.iter(|| {
            let res = backtest(
                strategy.clone(),
                &candles,
                config,
            );
            black_box(res.unwrap().ending_cash)
        })
    });
}

fn bench_backtest_10_instruments(c: &mut Criterion) {
    let candles = load_candles("market_data/spy_daily_5y.csv");
    
    // Simple momentum strategy
    let strategy = Strategy::builder("momentum_10x")
        .entry(IndicatorRef::sma(20).is_below(100.0))
        .exit(IndicatorRef::sma(20).is_above(100.0))
        .stop_loss(StopLoss::FixedPercent(2.0))
        .build()
        .unwrap();
    
    let config = BacktestConfig::default();
    
    c.bench_function("backtest_2yr_daily_10_instruments", |b| {
        b.iter(|| {
            // Simulate 10 sequential backtests
            let mut total_cash = 0.0;
            for _ in 0..10 {
                let res = backtest(
                    strategy.clone(),
                    &candles,
                    config,
                );
                total_cash += res.unwrap().ending_cash;
            }
            black_box(total_cash)
        })
    });
}

criterion_group!(benches, bench_backtest_1_instrument, bench_backtest_10_instruments);
criterion_main!(benches);
