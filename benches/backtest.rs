use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod common;
use common::load_candles;

fn bench_backtest(c: &mut Criterion) {
    let candles = load_candles("market_data/spy_daily_5y.csv");
    c.bench_function("backtest_stub", |b| {
        b.iter(|| {
            let mut cash = 100_000.0;
            let mut position = 0.0;
            // Simple rule: if close above previous close, stay long; else flat.
            let mut prev_close = None;
            for cndl in &candles {
                if let Some(prev) = prev_close {
                    if cndl.close > prev {
                        // enter long 1 unit if flat
                        if position == 0.0 {
                            position = 1.0;
                            cash -= cndl.close;
                        }
                    } else {
                        // exit to flat
                        if position > 0.0 {
                            cash += position * cndl.close;
                            position = 0.0;
                        }
                    }
                }
                prev_close = Some(cndl.close);
            }
            // liquidate at end
            cash += position * candles.last().unwrap().close;
            black_box(cash)
        })
    });
}

criterion_group!(backtest, bench_backtest);
criterion_main!(backtest);
