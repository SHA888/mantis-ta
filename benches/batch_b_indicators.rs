use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mantis_ta::indicators::{Ichimoku, Indicator};
use mantis_ta::types::Candle;

fn generate_candles(count: usize) -> Vec<Candle> {
    (0..count)
        .map(|i| {
            let price = 100.0 + (i as f64 * 0.1) + ((i as f64 * 0.05).sin() * 5.0);
            Candle {
                timestamp: i as i64 * 60_000,
                open: price * 0.99,
                high: price * 1.02,
                low: price * 0.98,
                close: price,
                volume: 1_000_000.0 + (i as f64 * 100.0),
            }
        })
        .collect()
}

fn bench_ichimoku_streaming(c: &mut Criterion) {
    c.bench_function("ichimoku_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut ichimoku = Ichimoku::new(9, 26, 52);
            for candle in &candles {
                let _ = ichimoku.next(candle);
            }
        });
    });
}

fn bench_ichimoku_batch(c: &mut Criterion) {
    c.bench_function("ichimoku_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let ichimoku = Ichimoku::new(9, 26, 52);
        b.iter(|| {
            let _ = ichimoku.calculate(&candles);
        });
    });
}

criterion_group!(benches, bench_ichimoku_streaming, bench_ichimoku_batch,);
criterion_main!(benches);
