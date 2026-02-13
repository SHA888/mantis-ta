use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mantis_ta::indicators::{
    BollingerBands, Indicator, PivotPoints, Stochastic, VolumeSMA, ATR, EMA, MACD, OBV, RSI, SMA,
};

mod common;
use common::load_candles;

fn bench_streaming(c: &mut Criterion) {
    let candles = load_candles("market_data/spy_daily_5y.csv");

    c.bench_function("sma_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = SMA::new(20);
                for cndl in &candles[..20] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[20].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("ema_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = EMA::new(20);
                for cndl in &candles[..20] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[20].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("macd_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = MACD::new(12, 26, 9);
                for cndl in &candles[..35] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[35].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("rsi_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = RSI::new(14);
                for cndl in &candles[..15] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[15].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("stoch_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = Stochastic::new(14, 3);
                for cndl in &candles[..17] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[17].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("bb_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = BollingerBands::new(20, 2.0);
                for cndl in &candles[..20] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[20].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("atr_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = ATR::new(14);
                for cndl in &candles[..14] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[14].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("volume_sma_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = VolumeSMA::new(20);
                for cndl in &candles[..20] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[20].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("obv_stream", |b| {
        b.iter_batched(
            || {
                let mut ind = OBV::new();
                for cndl in &candles[..1] {
                    let _ = ind.next(cndl);
                }
                (ind, candles[1].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("pivot_stream", |b| {
        b.iter_batched(
            || {
                let ind = PivotPoints::new();
                (ind, candles[0].clone())
            },
            |(mut ind, cndl)| black_box(ind.next(&cndl)),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_batch(c: &mut Criterion) {
    let candles = load_candles("market_data/spy_daily_5y.csv");

    c.bench_function("sma_batch", |b| {
        b.iter(|| black_box(SMA::new(20).calculate(&candles)))
    });
    c.bench_function("ema_batch", |b| {
        b.iter(|| black_box(EMA::new(20).calculate(&candles)))
    });
    c.bench_function("macd_batch", |b| {
        b.iter(|| black_box(MACD::new(12, 26, 9).calculate(&candles)))
    });
    c.bench_function("rsi_batch", |b| {
        b.iter(|| black_box(RSI::new(14).calculate(&candles)))
    });
    c.bench_function("stoch_batch", |b| {
        b.iter(|| black_box(Stochastic::new(14, 3).calculate(&candles)))
    });
    c.bench_function("bb_batch", |b| {
        b.iter(|| black_box(BollingerBands::new(20, 2.0).calculate(&candles)))
    });
    c.bench_function("atr_batch", |b| {
        b.iter(|| black_box(ATR::new(14).calculate(&candles)))
    });
    c.bench_function("volume_sma_batch", |b| {
        b.iter(|| black_box(VolumeSMA::new(20).calculate(&candles)))
    });
    c.bench_function("obv_batch", |b| {
        b.iter(|| black_box(OBV::new().calculate(&candles)))
    });
    c.bench_function("pivot_batch", |b| {
        b.iter(|| black_box(PivotPoints::new().calculate(&candles)))
    });
}

criterion_group!(indicators, bench_streaming, bench_batch);
criterion_main!(indicators);
