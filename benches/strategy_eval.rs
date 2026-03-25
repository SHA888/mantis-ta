use criterion::{Criterion, black_box, criterion_group, criterion_main};

use mantis_ta::strategy::evaluator::evaluate_strategy_batch;
use mantis_ta::strategy::indicator_ref::IndicatorRef;
use mantis_ta::strategy::types::{ConditionGroup, ConditionNode, StopLoss, Strategy};

mod common;
use common::load_candles;

fn bench_strategy_eval(c: &mut Criterion) {
    // Load once; keep only 2000 bars per TODO.
    let mut candles = load_candles("market_data/spy_daily_5y.csv");
    if candles.len() > 2000 {
        candles.truncate(2000);
    }

    // 5 conditions:
    // - SMA(5) > SMA(20)
    // - EMA(8) > EMA(21)
    // - RSI(14) < 70
    // - ATR(14) > 0
    // - SMA(1) > 0 (cheap extra condition)
    let entry = ConditionNode::Group(ConditionGroup::AllOf(vec![
        IndicatorRef::sma(5).is_above_indicator(IndicatorRef::sma(20)),
        IndicatorRef::ema(8).is_above_indicator(IndicatorRef::ema(21)),
        IndicatorRef::rsi(14).is_below(70.0),
        IndicatorRef::atr(14).is_above(0.0),
        IndicatorRef::sma(1).is_above(0.0),
    ]));

    let strategy = Strategy::builder("bench_strategy_eval")
        .entry(entry)
        .stop_loss(StopLoss::FixedPercent(2.0))
        .build()
        .unwrap();

    c.bench_function("strategy_eval_5cond_2000bars", |b| {
        b.iter(|| {
            let signals = evaluate_strategy_batch(black_box(&strategy), black_box(&candles));
            black_box(signals.len())
        })
    });
}

criterion_group!(strategy_eval, bench_strategy_eval);
criterion_main!(strategy_eval);
