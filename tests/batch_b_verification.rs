use mantis_ta::indicators::{AccumDist, Ichimoku, Indicator, KeltnerChannels, ParabolicSar, VWAP};
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

fn create_ohlcv_candles(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
) -> Vec<Candle> {
    highs
        .iter()
        .enumerate()
        .map(|(i, &high)| Candle {
            timestamp: i as i64 * 60_000,
            open: closes[i],
            high,
            low: lows[i],
            close: closes[i],
            volume: volumes[i],
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

#[test]
fn verify_parabolic_sar_reversal_clamps_to_current_bar_range() {
    // Bar 3 triggers an uptrend->downtrend reversal. The new SAR must be
    // clamped to at least the reversal bar's own high (and the prior bar's
    // high), matching TA-Lib's `sar = max(ep, todayHigh, yesterdayHigh)` —
    // it must never sit inside the bar's own high/low range.
    let candles = create_ohlc_candles(
        &[100.0, 102.0, 150.0],
        &[90.0, 92.0, 80.0],
        &[95.0, 97.0, 100.0],
    );
    let outputs = ParabolicSar::new(0.02, 0.02, 0.2).calculate(&candles);

    let reversal_sar = outputs[2].unwrap();
    assert!(
        reversal_sar >= candles[2].high,
        "SAR {reversal_sar} must be at or above the reversal bar's high {}",
        candles[2].high
    );
}

#[test]
fn verify_keltner_warmup_and_bounds() {
    let candles = trending_candles(60);
    let kc = KeltnerChannels::new(20, 10, 2.0);
    let outputs = kc.calculate(&candles);

    assert!(
        outputs
            .iter()
            .take(kc.warmup_period() - 1)
            .all(|o| o.is_none())
    );
    assert!(outputs[kc.warmup_period() - 1].is_some());

    for out in outputs.iter().flatten() {
        assert!(out.upper.is_finite());
        assert!(out.middle.is_finite());
        assert!(out.lower.is_finite());
        assert!(out.upper > out.middle);
        assert!(out.middle > out.lower);
    }
}

#[test]
fn verify_keltner_bands_formula_on_flat_input() {
    // Flat close (100) with constant high/low spread (110/90) settles to a
    // steady-state true range of 20 (|high-low| dominates once prev_close ==
    // close), so ATR converges to 20 and the middle band converges to close.
    let n = 40;
    let closes: Vec<f64> = vec![100.0; n];
    let highs: Vec<f64> = vec![110.0; n];
    let lows: Vec<f64> = vec![90.0; n];
    let candles = create_ohlc_candles(&highs, &lows, &closes);

    let outputs = KeltnerChannels::new(9, 9, 2.0).calculate(&candles);
    let last = outputs.last().unwrap().unwrap();

    assert!((last.middle - 100.0).abs() < 1e-9);
    assert!((last.upper - 140.0).abs() < 1e-9);
    assert!((last.lower - 60.0).abs() < 1e-9);
}

#[test]
fn verify_keltner_streaming_matches_batch() {
    let candles = trending_candles(70);
    let mut streaming = KeltnerChannels::new(20, 10, 2.0);
    let batch_outputs = KeltnerChannels::new(20, 10, 2.0).calculate(&candles);

    for (i, candle) in candles.iter().enumerate() {
        let streamed = streaming.next(candle);
        assert_eq!(streamed, batch_outputs[i], "mismatch at index {i}");
    }
}

#[test]
fn verify_keltner_reset_functionality() {
    let candles = trending_candles(60);
    let mut kc = KeltnerChannels::new(20, 10, 2.0);

    for candle in &candles {
        kc.next(candle);
    }
    assert!(kc.next(&candles[0]).is_some());

    kc.reset();
    assert_eq!(kc.next(&candles[0]), None);
}

#[test]
fn verify_vwap_warmup_and_window_bounds() {
    // VWAP is a volume-weighted average of typical price, so every output
    // must lie within [min typical price, max typical price] of its
    // trailing window regardless of the volume weighting.
    let period = 10;
    let n = 60;
    let closes: Vec<f64> = (0..n)
        .map(|i| 100.0 + (i as f64 * 0.37).sin() * 5.0)
        .collect();
    let highs: Vec<f64> = closes.iter().map(|c| c + 1.0).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 1.0).collect();
    let volumes: Vec<f64> = (0..n)
        .map(|i| 1_000.0 + (i as f64 * 37.0) % 5_000.0)
        .collect();
    let candles = create_ohlcv_candles(&highs, &lows, &closes, &volumes);

    let vwap = VWAP::new(period);
    let outputs = vwap.calculate(&candles);

    assert!(outputs.iter().take(period - 1).all(|o| o.is_none()));
    assert!(outputs[period - 1].is_some());

    for (i, out) in outputs.iter().enumerate() {
        let Some(v) = out else { continue };
        assert!(v.is_finite());
        let window = &closes[i + 1 - period..=i];
        let min_tp = window.iter().cloned().fold(f64::INFINITY, f64::min) - 1.0;
        let max_tp = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max) + 1.0;
        assert!(
            *v >= min_tp && *v <= max_tp,
            "VWAP {v} at index {i} outside window typical-price bounds [{min_tp}, {max_tp}]"
        );
    }
}

#[test]
fn verify_vwap_reduces_to_typical_price_average_under_constant_volume() {
    // When volume is constant across the window, the volume weighting
    // cancels out and VWAP must equal the plain average of typical prices.
    let n = 20;
    let closes: Vec<f64> = (0..n).map(|i| 50.0 + i as f64).collect();
    let highs: Vec<f64> = closes.iter().map(|c| c + 2.0).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 2.0).collect();
    let volumes: Vec<f64> = vec![1_000.0; n];
    let candles = create_ohlcv_candles(&highs, &lows, &closes, &volumes);

    let period = 5;
    let outputs = VWAP::new(period).calculate(&candles);

    for i in (period - 1)..n {
        let window_tp_sum: f64 = closes[i + 1 - period..=i].iter().sum();
        let expected = window_tp_sum / period as f64;
        assert!((outputs[i].unwrap() - expected).abs() < 1e-9);
    }
}

#[test]
fn verify_vwap_streaming_matches_batch() {
    let n = 50;
    let closes: Vec<f64> = (0..n)
        .map(|i| 100.0 + (i as f64 * 0.21).cos() * 4.0)
        .collect();
    let highs: Vec<f64> = closes.iter().map(|c| c + 1.5).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 1.5).collect();
    let volumes: Vec<f64> = (0..n)
        .map(|i| 2_000.0 + (i as f64 * 53.0) % 3_000.0)
        .collect();
    let candles = create_ohlcv_candles(&highs, &lows, &closes, &volumes);

    let mut streaming = VWAP::new(14);
    let batch_outputs = VWAP::new(14).calculate(&candles);

    for (i, candle) in candles.iter().enumerate() {
        let streamed = streaming.next(candle);
        assert_eq!(streamed, batch_outputs[i], "mismatch at index {i}");
    }
}

#[test]
fn verify_vwap_reset_functionality() {
    let candles = trending_candles(30);
    let mut vwap = VWAP::new(10);

    for candle in &candles {
        vwap.next(candle);
    }
    assert!(vwap.next(&candles[0]).is_some());

    vwap.reset();
    assert_eq!(vwap.next(&candles[0]), None);
}

// Formula/manual-calculation parity for A/D is covered by the inline unit test
// in `accum_dist.rs` and, over a 2000-bar series, by the native TA-Lib parity
// test `verify_accum_dist` in `tests/indicator_verification`. The tests below
// cover behaviors those don't: boundary bars, zero-range no-op, streaming/batch
// equivalence, and reset.
#[test]
fn verify_accum_dist_close_at_high_accumulates_full_volume() {
    // Close pinned to the bar's high -> money flow multiplier == 1, so AD
    // becomes the running sum of volume (strict accumulation).
    let n = 10;
    let volumes: Vec<f64> = (0..n).map(|i| 100.0 + i as f64 * 10.0).collect();
    let highs = vec![110.0; n];
    let lows = vec![90.0; n];
    let closes = vec![110.0; n];
    let candles = create_ohlcv_candles(&highs, &lows, &closes, &volumes);

    let outputs = AccumDist::new().calculate(&candles);
    let expected_total: f64 = volumes.iter().sum();
    assert!((outputs.last().unwrap().unwrap() - expected_total).abs() < 1e-9);
}

#[test]
fn verify_accum_dist_close_at_low_distributes_full_volume() {
    // Close pinned to the bar's low -> money flow multiplier == -1, so AD
    // becomes the running negative sum of volume (strict distribution).
    let n = 10;
    let volumes: Vec<f64> = (0..n).map(|i| 100.0 + i as f64 * 10.0).collect();
    let highs = vec![110.0; n];
    let lows = vec![90.0; n];
    let closes = vec![90.0; n];
    let candles = create_ohlcv_candles(&highs, &lows, &closes, &volumes);

    let outputs = AccumDist::new().calculate(&candles);
    let expected_total: f64 = -volumes.iter().sum::<f64>();
    assert!((outputs.last().unwrap().unwrap() - expected_total).abs() < 1e-9);
}

#[test]
fn verify_accum_dist_zero_range_bar_is_a_no_op() {
    // High == Low (zero range) must not divide by zero and must contribute
    // nothing to the running total (TA-Lib's AD guards this case as 0).
    let candles = create_ohlcv_candles(&[100.0], &[100.0], &[100.0], &[5_000.0]);
    let outputs = AccumDist::new().calculate(&candles);
    assert!((outputs[0].unwrap() - 0.0).abs() < 1e-9);
}

#[test]
fn verify_accum_dist_streaming_matches_batch() {
    let candles = trending_candles(60);
    let mut streaming = AccumDist::new();
    let batch_outputs = AccumDist::new().calculate(&candles);

    for (i, candle) in candles.iter().enumerate() {
        let streamed = streaming.next(candle);
        assert_eq!(streamed, batch_outputs[i], "mismatch at index {i}");
    }
}

#[test]
fn verify_accum_dist_reset_functionality() {
    // trending_candles() centers close exactly between high/low (CLV == 0
    // every bar), so use a series where close is offset toward the high to
    // exercise real accumulation.
    let n = 20;
    let closes: Vec<f64> = (0..n).map(|i| 100.0 + i as f64).collect();
    let highs: Vec<f64> = closes.iter().map(|c| c + 1.0).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 3.0).collect();
    let candles = create_ohlc_candles(&highs, &lows, &closes);
    let mut ad = AccumDist::new();

    for candle in &candles {
        ad.next(candle);
    }
    let accumulated = ad.next(&candles[0]).unwrap();
    assert!(accumulated != 0.0);

    ad.reset();
    let mut fresh = AccumDist::new();
    assert_eq!(ad.next(&candles[0]), fresh.next(&candles[0]));
}
