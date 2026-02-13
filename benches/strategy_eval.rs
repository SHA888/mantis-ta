use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod common;
use common::load_candles;

fn bench_strategy_eval(c: &mut Criterion) {
    let candles = load_candles("market_data/spy_daily_5y.csv");
    c.bench_function("strategy_eval_stub", |b| {
        b.iter(|| {
            // Simple signal sweep: count closes above SMA(20) computed once.
            let mut count = 0usize;
            let mut sum = 0.0;
            let mut window = Vec::with_capacity(20);
            for cndl in &candles {
                let close = cndl.close;
                window.push(close);
                sum += close;
                if window.len() > 20 {
                    sum -= window.remove(0);
                }
                if window.len() == 20 {
                    let sma = sum / 20.0;
                    if close > sma {
                        count += 1;
                    }
                }
            }
            black_box(count)
        })
    });
}

criterion_group!(strategy_eval, bench_strategy_eval);
criterion_main!(strategy_eval);
