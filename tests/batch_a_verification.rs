use mantis_ta::indicators::{ADX, CCI, DEMA, Indicator, ROC, StdDev, TEMA, WMA, WilliamsR};
use mantis_ta::types::Candle;

fn create_candles(prices: &[f64]) -> Vec<Candle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &price)| Candle {
            timestamp: i as i64 * 60_000,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1_000_000.0,
        })
        .collect()
}

fn create_ohlc_candles(opens: &[f64], highs: &[f64], lows: &[f64], closes: &[f64]) -> Vec<Candle> {
    opens
        .iter()
        .enumerate()
        .map(|(i, &open)| Candle {
            timestamp: i as i64 * 60_000,
            open,
            high: highs[i],
            low: lows[i],
            close: closes[i],
            volume: 1_000_000.0,
        })
        .collect()
}

#[test]
fn verify_wma_consistency() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00,
    ];
    let candles = create_candles(&prices);

    let wma = WMA::new(5);
    let outputs = wma.calculate(&candles);

    assert_eq!(outputs[0], None);
    assert_eq!(outputs[1], None);
    assert_eq!(outputs[2], None);
    assert_eq!(outputs[3], None);
    assert!(outputs[4].is_some());

    let val = outputs[4].unwrap();
    assert!(val > 0.0 && val < 100.0);
}

#[test]
fn verify_dema_consistency() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00,
    ];
    let candles = create_candles(&prices);

    let dema = DEMA::new(5);
    let outputs = dema.calculate(&candles);

    let warmup = dema.warmup_period();
    assert!(warmup > 0);

    let mut has_output = false;
    for output in &outputs {
        if output.is_some() {
            has_output = true;
            let val = output.unwrap();
            assert!(val > 0.0 && val < 100.0);
        }
    }
    assert!(has_output, "DEMA should produce at least one output");
}

#[test]
fn verify_tema_consistency() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00,
        46.00, 46.00,
    ];
    let candles = create_candles(&prices);

    let tema = TEMA::new(5);
    let outputs = tema.calculate(&candles);

    let warmup = tema.warmup_period();
    assert!(warmup > 0);

    let mut has_output = false;
    for output in &outputs {
        if output.is_some() {
            has_output = true;
            let val = output.unwrap();
            assert!(val > 0.0 && val < 100.0);
        }
    }
    assert!(has_output, "TEMA should produce at least one output");
}

#[test]
fn verify_roc_consistency() {
    let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0];
    let candles = create_candles(&prices);

    let roc = ROC::new(2);
    let outputs = roc.calculate(&candles);

    assert_eq!(outputs[0], None);
    assert!(outputs[1].is_some());

    let roc_val = outputs[1].unwrap();
    assert!((roc_val - 2.0).abs() < 0.0001);
}

#[test]
fn verify_stddev_consistency() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.00, 46.00, 46.00, 46.00, 46.00,
    ];
    let candles = create_candles(&prices);

    let stddev = StdDev::new(5);
    let outputs = stddev.calculate(&candles);

    for i in 0..4 {
        assert_eq!(outputs[i], None);
    }
    assert!(outputs[4].is_some());

    let val = outputs[4].unwrap();
    assert!(val >= 0.0);
}

#[test]
fn verify_cci_consistency() {
    let highs = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
    ];
    let lows = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
    ];
    let closes = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
    ];
    let candles = create_ohlc_candles(&closes, &highs, &lows, &closes);

    let cci = CCI::new(5);
    let outputs = cci.calculate(&candles);

    for i in 0..4 {
        assert_eq!(outputs[i], None);
    }
    assert!(outputs[4].is_some());
}

#[test]
fn verify_williams_r_consistency() {
    let highs = vec![105.0, 106.0, 107.0, 108.0, 109.0, 110.0];
    let lows = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0];
    let closes = vec![102.0, 103.0, 104.0, 105.0, 106.0, 107.0];
    let candles = create_ohlc_candles(&closes, &highs, &lows, &closes);

    let wr = WilliamsR::new(3);
    let outputs = wr.calculate(&candles);

    assert_eq!(outputs[0], None);
    assert_eq!(outputs[1], None);
    assert!(outputs[2].is_some());

    let val = outputs[2].unwrap();
    assert!(val >= -100.0 && val <= 0.0);
}

#[test]
fn verify_adx_consistency() {
    let highs = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.50, 46.75, 47.00, 47.25, 47.50, 47.75, 48.00, 48.25, 48.50, 48.75, 49.00,
        49.25, 49.50,
    ];
    let lows = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.25, 46.50, 46.75, 47.00, 47.25, 47.50, 47.75, 48.00, 48.25, 48.50, 48.75,
        49.00, 49.25,
    ];
    let closes = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00, 46.38, 46.63, 46.88, 47.13, 47.38, 47.63, 47.88, 48.13, 48.38, 48.63, 48.88,
        49.13, 49.38,
    ];
    let candles = create_ohlc_candles(&closes, &highs, &lows, &closes);

    let adx = ADX::new(7);
    let outputs = adx.calculate(&candles);

    let warmup = adx.warmup_period();
    assert_eq!(warmup, 14);

    for i in 0..warmup {
        assert_eq!(outputs[i], None);
    }
    assert!(outputs[warmup].is_some());

    let output = outputs[warmup].unwrap();
    assert!(output.adx >= 0.0 && output.adx <= 100.0);
    assert!(output.plus_di >= 0.0);
    assert!(output.minus_di >= 0.0);
}

#[test]
fn verify_streaming_equals_batch() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.00,
    ];
    let candles = create_candles(&prices);

    let wma = WMA::new(5);
    let batch_outputs = wma.calculate(&candles);

    let mut streaming_wma = WMA::new(5);
    let mut streaming_outputs = Vec::new();
    for candle in &candles {
        streaming_outputs.push(streaming_wma.next(candle));
    }

    for (i, (batch, streaming)) in batch_outputs
        .iter()
        .zip(streaming_outputs.iter())
        .enumerate()
    {
        match (batch, streaming) {
            (Some(b), Some(s)) => {
                assert!(
                    (b - s).abs() < 1e-10,
                    "Mismatch at index {}: batch={}, streaming={}",
                    i,
                    b,
                    s
                );
            }
            (None, None) => {}
            _ => panic!(
                "Mismatch at index {}: batch={:?}, streaming={:?}",
                i, batch, streaming
            ),
        }
    }
}

#[test]
fn verify_reset_functionality() {
    let prices = vec![44.34, 44.09, 44.15, 43.61, 44.33];
    let candles = create_candles(&prices);

    let mut wma = WMA::new(3);

    for candle in &candles {
        wma.next(candle);
    }

    assert!(wma.next(&candles[0]).is_some());

    wma.reset();

    assert_eq!(wma.next(&candles[0]), None);
}
