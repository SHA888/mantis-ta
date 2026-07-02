use mantis_ta::indicators::{Ichimoku, Indicator, ParabolicSar};
use mantis_ta::types::Candle;

fn create_ohlc_candles(highs: &[f64], lows: &[f64], closes: &[f64]) -> Vec<Candle> {
    highs
        .iter()
        .enumerate()
        .map(|(i, &high)| Candle {
            timestamp: i as i64 * 60_000,
            open: closes[i],
            high,
            low: lows[i],
            close: closes[i],
            volume: 1_000_000.0,
        })
        .collect()
}

fn trending_candles(n: usize) -> Vec<Candle> {
    let closes: Vec<f64> = (0..n).map(|i| 100.0 + i as f64).collect();
    let highs: Vec<f64> = closes.iter().map(|c| c + 1.0).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 1.0).collect();
    create_ohlc_candles(&highs, &lows, &closes)
}

fn up_then_down_candles(up_n: usize, down_n: usize) -> Vec<Candle> {
    let mut closes = Vec::with_capacity(up_n + down_n);
    let mut price = 100.0;
    for _ in 0..up_n {
        price += 1.0;
        closes.push(price);
    }
    for _ in 0..down_n {
        price -= 2.0;
        closes.push(price);
    }
    let highs: Vec<f64> = closes.iter().map(|c| c + 0.5).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 0.5).collect();
    create_ohlc_candles(&highs, &lows, &closes)
}

#[test]
fn verify_ichimoku_warmup_and_bounds() {
    let candles = trending_candles(60);
    let ichimoku = Ichimoku::new(9, 26, 52);
    let outputs = ichimoku.calculate(&candles);

    assert!(
        outputs
            .iter()
            .take(ichimoku.warmup_period() - 1)
            .all(|o| o.is_none())
    );
    assert!(outputs[ichimoku.warmup_period() - 1].is_some());

    for out in outputs.iter().flatten() {
        assert!(out.tenkan_sen.is_finite());
        assert!(out.kijun_sen.is_finite());
        assert!(out.senkou_span_a.is_finite());
        assert!(out.senkou_span_b.is_finite());
        assert!(out.chikou_span.is_finite());
    }
}

#[test]
fn verify_ichimoku_tenkan_kijun_formula() {
    // Flat high/low window: tenkan/kijun midpoint == the flat high/low midpoint.
    let n = 60;
    let closes: Vec<f64> = vec![100.0; n];
    let highs: Vec<f64> = vec![110.0; n];
    let lows: Vec<f64> = vec![90.0; n];
    let candles = create_ohlc_candles(&highs, &lows, &closes);

    let ichimoku = Ichimoku::new(9, 26, 52);
    let outputs = ichimoku.calculate(&candles);
    let last = outputs.last().unwrap().unwrap();

    assert!((last.tenkan_sen - 100.0).abs() < 1e-9);
    assert!((last.kijun_sen - 100.0).abs() < 1e-9);
    assert!((last.senkou_span_a - 100.0).abs() < 1e-9);
    assert!((last.senkou_span_b - 100.0).abs() < 1e-9);
    assert!((last.chikou_span - 100.0).abs() < 1e-9);
}

#[test]
fn verify_ichimoku_streaming_matches_batch() {
    let candles = trending_candles(70);
    let mut streaming = Ichimoku::new(9, 26, 52);
    let batch_outputs = Ichimoku::new(9, 26, 52).calculate(&candles);

    for (i, candle) in candles.iter().enumerate() {
        let streamed = streaming.next(candle);
        assert_eq!(streamed, batch_outputs[i], "mismatch at index {i}");
    }
}

#[test]
fn verify_ichimoku_reset_functionality() {
    let candles = trending_candles(60);
    let mut ichimoku = Ichimoku::new(9, 26, 52);

    for candle in &candles {
        ichimoku.next(candle);
    }
    assert!(ichimoku.next(&candles[0]).is_some());

    ichimoku.reset();
    assert_eq!(ichimoku.next(&candles[0]), None);
}

#[test]
fn verify_parabolic_sar_warmup_and_bounds() {
    let candles = trending_candles(60);
    let sar = ParabolicSar::new(0.02, 0.02, 0.2);
    let outputs = sar.calculate(&candles);

    assert_eq!(sar.warmup_period(), 2);
    assert!(outputs[0].is_none());
    assert!(outputs[1].is_some());
    assert!(outputs.iter().skip(1).all(|v| v.is_some()));

    // Sustained uptrend: SAR must trail below price (never above the bar's low).
    for (out, candle) in outputs.iter().zip(candles.iter()).skip(1) {
        let v = out.unwrap();
        assert!(v.is_finite());
        assert!(v <= candle.low);
    }
}

#[test]
fn verify_parabolic_sar_reversal_flips_side() {
    let candles = up_then_down_candles(30, 30);
    let outputs = ParabolicSar::new(0.02, 0.02, 0.2).calculate(&candles);

    // Early in the uptrend leg, SAR trails below price.
    let early = outputs[5].unwrap();
    assert!(early <= candles[5].low);

    // After a steep, sustained decline, SAR must have flipped above price.
    let last = outputs.last().unwrap().unwrap();
    assert!(last >= candles.last().unwrap().high);
}

#[test]
fn verify_parabolic_sar_streaming_matches_batch() {
    let candles = up_then_down_candles(20, 20);
    let batch = ParabolicSar::new(0.02, 0.02, 0.2).calculate(&candles);

    let mut streaming_sar = ParabolicSar::new(0.02, 0.02, 0.2);
    let streaming: Vec<Option<f64>> = candles.iter().map(|c| streaming_sar.next(c)).collect();

    assert_eq!(batch, streaming);
}

#[test]
fn verify_parabolic_sar_reset_functionality() {
    let candles = trending_candles(10);
    let mut sar = ParabolicSar::new(0.02, 0.02, 0.2);

    assert!(sar.next(&candles[0]).is_none());
    assert!(sar.next(&candles[1]).is_some());

    sar.reset();
    assert_eq!(sar.next(&candles[0]), None);
}
