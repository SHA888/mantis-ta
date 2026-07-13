use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mantis_ta::indicators::{
    AccumDist, DonchianChannels, Ichimoku, Indicator, KeltnerChannels, MFI, ParabolicSar, VWAP,
};
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

fn bench_parabolic_sar_streaming(c: &mut Criterion) {
    c.bench_function("parabolic_sar_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut sar = ParabolicSar::new(0.02, 0.02, 0.2);
            for candle in &candles {
                let _ = sar.next(candle);
            }
        });
    });
}

fn bench_parabolic_sar_batch(c: &mut Criterion) {
    c.bench_function("parabolic_sar_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let sar = ParabolicSar::new(0.02, 0.02, 0.2);
        b.iter(|| {
            let _ = sar.calculate(&candles);
        });
    });
}

fn bench_mfi_streaming(c: &mut Criterion) {
    c.bench_function("mfi_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut mfi = MFI::new(14);
            for candle in &candles {
                let _ = mfi.next(candle);
            }
        });
    });
}

fn bench_mfi_batch(c: &mut Criterion) {
    c.bench_function("mfi_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let mfi = MFI::new(14);
        b.iter(|| {
            let _ = mfi.calculate(&candles);
        });
    });
}

fn bench_keltner_streaming(c: &mut Criterion) {
    c.bench_function("keltner_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut kc = KeltnerChannels::new(20, 10, 2.0);
            for candle in &candles {
                let _ = kc.next(candle);
            }
        });
    });
}

fn bench_keltner_batch(c: &mut Criterion) {
    c.bench_function("keltner_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let kc = KeltnerChannels::new(20, 10, 2.0);
        b.iter(|| {
            let _ = kc.calculate(&candles);
        });
    });
}

fn bench_vwap_streaming(c: &mut Criterion) {
    c.bench_function("vwap_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut vwap = VWAP::new(20);
            for candle in &candles {
                let _ = vwap.next(candle);
            }
        });
    });
}

fn bench_vwap_batch(c: &mut Criterion) {
    c.bench_function("vwap_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let vwap = VWAP::new(20);
        b.iter(|| {
            let _ = vwap.calculate(&candles);
        });
    });
}

fn bench_accum_dist_streaming(c: &mut Criterion) {
    c.bench_function("accum_dist_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut ad = AccumDist::new();
            for candle in &candles {
                let _ = ad.next(candle);
            }
        });
    });
}

fn bench_accum_dist_batch(c: &mut Criterion) {
    c.bench_function("accum_dist_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let ad = AccumDist::new();
        b.iter(|| {
            let _ = ad.calculate(&candles);
        });
    });
}

fn bench_donchian_streaming(c: &mut Criterion) {
    c.bench_function("donchian_streaming_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        b.iter(|| {
            let mut dc = DonchianChannels::new(20);
            for candle in &candles {
                let _ = dc.next(candle);
            }
        });
    });
}

fn bench_donchian_batch(c: &mut Criterion) {
    c.bench_function("donchian_batch_252_bars", |b| {
        let candles = black_box(generate_candles(252));
        let dc = DonchianChannels::new(20);
        b.iter(|| {
            let _ = dc.calculate(&candles);
        });
    });
}

criterion_group!(
    benches,
    bench_ichimoku_streaming,
    bench_ichimoku_batch,
    bench_parabolic_sar_streaming,
    bench_parabolic_sar_batch,
    bench_mfi_streaming,
    bench_mfi_batch,
    bench_keltner_streaming,
    bench_keltner_batch,
    bench_vwap_streaming,
    bench_vwap_batch,
    bench_accum_dist_streaming,
    bench_accum_dist_batch,
    bench_donchian_streaming,
    bench_donchian_batch,
);
criterion_main!(benches);
