use mantis_ta::indicators::momentum::Stochastic;
use mantis_ta::indicators::volatility::ATR;
use mantis_ta::indicators::{
    BollingerBands, Indicator, PivotPoints, VolumeSMA, EMA, MACD, OBV, RSI, SMA,
};
use mantis_ta::types::Candle;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn random_candles(len: usize, seed: u64) -> Vec<Candle> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut price = 100.0;
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let drift: f64 = rng.gen_range(-0.5..0.5);
        let shock: f64 = rng.gen_range(-0.5..0.5);
        price = (price + drift + shock).max(0.01);
        let high = price + rng.gen_range(0.0..0.5);
        let low = (price - rng.gen_range(0.0..0.5)).max(0.0);
        let open = (price + rng.gen_range(-0.25..0.25)).clamp(low, high);
        let close = price.clamp(low, high);
        let volume = rng.gen_range(500.0..1500.0);
        out.push(Candle {
            timestamp: i as i64,
            open,
            high,
            low,
            close,
            volume,
        });
    }
    out
}

#[test]
fn rsi_within_bounds() {
    let candles = random_candles(200, 42);
    let rsi_vals: Vec<_> = RSI::new(14).calculate(&candles);
    for v in rsi_vals.iter().flatten() {
        assert!((0.0..=100.0).contains(v), "RSI out of bounds: {}", v);
    }
}

#[test]
fn bollinger_middle_matches_sma() {
    let candles = random_candles(200, 7);
    let bb_vals: Vec<_> = BollingerBands::new(20, 2.0).calculate(&candles);
    let sma_vals: Vec<_> = SMA::new(20).calculate(&candles);
    assert_eq!(bb_vals.len(), sma_vals.len());
    for (i, (bb, sma)) in bb_vals.iter().zip(sma_vals.iter()).enumerate() {
        match (bb, sma) {
            (Some(b), Some(s)) => assert!((b.middle - s).abs() < 1e-9, "BB mid != SMA at {}", i),
            (None, None) => {}
            _ => panic!("Option mismatch at {}", i),
        }
    }
}

#[test]
fn streaming_matches_batch_sma() {
    let candles = random_candles(120, 99);
    let mut sma = SMA::new(10);
    let mut stream_out = Vec::new();
    for c in &candles {
        stream_out.push(sma.next(c));
    }
    let batch_out = SMA::new(10).calculate(&candles);
    assert_eq!(stream_out, batch_out);
}

#[test]
fn fuzz_no_panic_core_indicators() {
    let candles = random_candles(150, 2024);
    let mut sma = SMA::new(10);
    let mut ema = EMA::new(10);
    let mut rsi = RSI::new(14);
    let mut bb = BollingerBands::new(20, 2.0);
    let mut atr = ATR::new(14);
    let mut stoch = Stochastic::new(14, 3);
    let mut obv = OBV::new();
    let mut vol_sma = VolumeSMA::new(20);
    let mut macd = MACD::new(12, 26, 9);
    let mut pivot = PivotPoints::new();

    for c in &candles {
        let _ = sma.next(c);
        let _ = ema.next(c);
        let _ = rsi.next(c);
        let _ = bb.next(c);
        let _ = atr.next(c);
        let _ = stoch.next(c);
        let _ = obv.next(c);
        let _ = vol_sma.next(c);
        let _ = macd.next(c);
        let _ = pivot.next(c);
    }
}
