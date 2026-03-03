use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mantis_ta::indicators::{Indicator, CCI, DEMA, ROC, StdDev, TEMA, WMA, WilliamsR, ADX};
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

fn bench_wma_streaming(c: &mut Criterion) {
    c.bench_function("wma_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut wma = WMA::new(20);
            for candle in &candles {
                let _ = wma.next(candle);
            }
        });
    });
}

fn bench_wma_batch(c: &mut Criterion) {
    c.bench_function("wma_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let wma = WMA::new(20);
        b.iter(|| {
            let _ = wma.calculate(&candles);
        });
    });
}

fn bench_dema_streaming(c: &mut Criterion) {
    c.bench_function("dema_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut dema = DEMA::new(10);
            for candle in &candles {
                let _ = dema.next(candle);
            }
        });
    });
}

fn bench_dema_batch(c: &mut Criterion) {
    c.bench_function("dema_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let dema = DEMA::new(10);
        b.iter(|| {
            let _ = dema.calculate(&candles);
        });
    });
}

fn bench_tema_streaming(c: &mut Criterion) {
    c.bench_function("tema_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut tema = TEMA::new(10);
            for candle in &candles {
                let _ = tema.next(candle);
            }
        });
    });
}

fn bench_tema_batch(c: &mut Criterion) {
    c.bench_function("tema_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let tema = TEMA::new(10);
        b.iter(|| {
            let _ = tema.calculate(&candles);
        });
    });
}

fn bench_roc_streaming(c: &mut Criterion) {
    c.bench_function("roc_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut roc = ROC::new(12);
            for candle in &candles {
                let _ = roc.next(candle);
            }
        });
    });
}

fn bench_roc_batch(c: &mut Criterion) {
    c.bench_function("roc_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let roc = ROC::new(12);
        b.iter(|| {
            let _ = roc.calculate(&candles);
        });
    });
}

fn bench_stddev_streaming(c: &mut Criterion) {
    c.bench_function("stddev_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut stddev = StdDev::new(20);
            for candle in &candles {
                let _ = stddev.next(candle);
            }
        });
    });
}

fn bench_stddev_batch(c: &mut Criterion) {
    c.bench_function("stddev_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let stddev = StdDev::new(20);
        b.iter(|| {
            let _ = stddev.calculate(&candles);
        });
    });
}

fn bench_cci_streaming(c: &mut Criterion) {
    c.bench_function("cci_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut cci = CCI::new(20);
            for candle in &candles {
                let _ = cci.next(candle);
            }
        });
    });
}

fn bench_cci_batch(c: &mut Criterion) {
    c.bench_function("cci_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let cci = CCI::new(20);
        b.iter(|| {
            let _ = cci.calculate(&candles);
        });
    });
}

fn bench_williams_r_streaming(c: &mut Criterion) {
    c.bench_function("williams_r_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut wr = WilliamsR::new(14);
            for candle in &candles {
                let _ = wr.next(candle);
            }
        });
    });
}

fn bench_williams_r_batch(c: &mut Criterion) {
    c.bench_function("williams_r_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let wr = WilliamsR::new(14);
        b.iter(|| {
            let _ = wr.calculate(&candles);
        });
    });
}

fn bench_adx_streaming(c: &mut Criterion) {
    c.bench_function("adx_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut adx = ADX::new(14);
            for candle in &candles {
                let _ = adx.next(candle);
            }
        });
    });
}

fn bench_adx_batch(c: &mut Criterion) {
    c.bench_function("adx_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let adx = ADX::new(14);
        b.iter(|| {
            let _ = adx.calculate(&candles);
        });
    });
}

criterion_group!(
    benches,
    bench_wma_streaming,
    bench_wma_batch,
    bench_dema_streaming,
    bench_dema_batch,
    bench_tema_streaming,
    bench_tema_batch,
    bench_roc_streaming,
    bench_roc_batch,
    bench_stddev_streaming,
    bench_stddev_batch,
    bench_cci_streaming,
    bench_cci_batch,
    bench_williams_r_streaming,
    bench_williams_r_batch,
    bench_adx_streaming,
    bench_adx_batch,
);
criterion_main!(benches);
