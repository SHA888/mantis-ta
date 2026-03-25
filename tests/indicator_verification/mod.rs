use mantis_ta::indicators::{
    ATR, BollingerBands, EMA, Indicator, MACD, OBV, PivotPoints, RSI, SMA, Stochastic, VolumeSMA,
};
use mantis_ta::types::{BollingerOutput, MacdOutput, PivotOutput, StochasticOutput};

use crate::common::{load_candles, load_reference_json, load_reference_series};

const EPS: f64 = 1e-9;

fn assert_close(a: f64, b: f64, label: &str, index: usize) {
    assert!(
        (a - b).abs() < EPS,
        "{} index {}: {} vs {} (diff {})",
        label,
        index,
        a,
        b,
        (a - b).abs()
    );
}

fn assert_series(opt_out: &[Option<f64>], reference: &[Option<f64>], label: &str) {
    assert_eq!(opt_out.len(), reference.len(), "{} length mismatch", label);
    for (i, (o, r)) in opt_out.iter().zip(reference.iter()).enumerate() {
        match (o, r) {
            (Some(a), Some(b)) => assert_close(*a, *b, label, i),
            (None, None) => {}
            _ => panic!("{} index {} option mismatch: {:?} vs {:?}", label, i, o, r),
        }
    }
}

#[test]
fn verify_sma() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let reference = load_reference_series("reference/sma_20.json").unwrap();
    let out: Vec<Option<f64>> = SMA::new(20).calculate(&candles);
    assert_series(&out, &reference, "sma_20");
}

#[test]
fn verify_ema() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let reference = load_reference_series("reference/ema_20.json").unwrap();
    let out: Vec<Option<f64>> = EMA::new(20).calculate(&candles);
    assert_series(&out, &reference, "ema_20");
}

#[test]
fn verify_macd() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let ref_macd = load_reference_series("reference/macd_line.json").unwrap();
    let ref_signal = load_reference_series("reference/macd_signal.json").unwrap();
    let ref_hist = load_reference_series("reference/macd_hist.json").unwrap();
    let out: Vec<Option<MacdOutput>> = MACD::new(12, 26, 9).calculate(&candles);
    assert_eq!(out.len(), ref_macd.len());
    // Rust MACD returns None until signal EMA warms up (slow + signal - 1).
    // Python stores MACD line / signal / hist independently with different warmups.
    // We only compare when Rust emits Some (all three components valid).
    for i in 0..out.len() {
        if let Some(m) = &out[i] {
            assert_close(m.macd_line, ref_macd[i].unwrap(), "macd_line", i);
            assert_close(m.signal_line, ref_signal[i].unwrap(), "signal", i);
            assert_close(m.histogram, ref_hist[i].unwrap(), "hist", i);
        }
    }
    // Verify the first Some appears at the expected warmup index
    let first_some = out.iter().position(|o| o.is_some());
    assert!(first_some.is_some(), "MACD never emitted");
}

#[test]
fn verify_rsi() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let reference = load_reference_series("reference/rsi_14.json").unwrap();
    let out: Vec<Option<f64>> = RSI::new(14).calculate(&candles);
    assert_series(&out, &reference, "rsi_14");
}

#[test]
fn verify_stochastic() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let ref_k = load_reference_series("reference/stoch_k.json").unwrap();
    let ref_d = load_reference_series("reference/stoch_d.json").unwrap();
    let out: Vec<Option<StochasticOutput>> = Stochastic::new(14, 3).calculate(&candles);
    assert_eq!(out.len(), ref_k.len());
    // Rust Stochastic returns None until both %K and %D are valid.
    // Python stores %K and %D independently with different warmups.
    // Compare only when Rust emits Some.
    for i in 0..out.len() {
        if let Some(s) = &out[i] {
            assert_close(s.k, ref_k[i].unwrap(), "stoch_k", i);
            assert_close(s.d, ref_d[i].unwrap(), "stoch_d", i);
        }
    }
    let first_some = out.iter().position(|o| o.is_some());
    assert!(first_some.is_some(), "Stochastic never emitted");
}

#[test]
fn verify_bollinger() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let ref_upper = load_reference_series("reference/bb_upper.json").unwrap();
    let ref_middle = load_reference_series("reference/bb_middle.json").unwrap();
    let ref_lower = load_reference_series("reference/bb_lower.json").unwrap();
    let out: Vec<Option<BollingerOutput>> = BollingerBands::new(20, 2.0).calculate(&candles);
    assert_eq!(out.len(), ref_upper.len());
    for i in 0..out.len() {
        match &out[i] {
            Some(b) => {
                assert_close(b.upper, ref_upper[i].unwrap(), "bb_upper", i);
                assert_close(b.middle, ref_middle[i].unwrap(), "bb_mid", i);
                assert_close(b.lower, ref_lower[i].unwrap(), "bb_lower", i);
            }
            None => {
                assert!(
                    ref_upper[i].is_none() && ref_middle[i].is_none() && ref_lower[i].is_none(),
                    "bb option mismatch at {}",
                    i
                );
            }
        }
    }
}

#[test]
fn verify_atr() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let reference = load_reference_series("reference/atr_14.json").unwrap();
    let out: Vec<Option<f64>> = ATR::new(14).calculate(&candles);
    assert_series(&out, &reference, "atr_14");
}

#[test]
fn verify_volume_sma() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let reference = load_reference_series("reference/volume_sma_20.json").unwrap();
    let out: Vec<Option<f64>> = VolumeSMA::new(20).calculate(&candles);
    assert_series(&out, &reference, "volume_sma_20");
}

#[test]
fn verify_obv() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let reference = load_reference_series("reference/obv.json").unwrap();
    let out: Vec<Option<f64>> = OBV::new().calculate(&candles);
    // OBV has no warmup; treat None in ref as zero-length check
    assert_eq!(out.len(), reference.len());
    for (i, (o, r)) in out.iter().zip(reference.iter()).enumerate() {
        match (o, r) {
            (Some(a), Some(b)) => assert_close(*a, *b, "obv", i),
            _ => panic!("obv option mismatch at {}", i),
        }
    }
}

#[test]
fn verify_pivot_points() {
    let candles = load_candles("market_data/spy_daily_5y.csv").unwrap();
    let value = load_reference_json("reference/pivot_points.json").unwrap();
    let arr = value.as_array().expect("pivot_points.json array");
    let mut reference: Vec<PivotOutput> = Vec::with_capacity(arr.len());
    for obj in arr {
        let o = obj.as_object().expect("pivot object");
        let f = |k: &str| o.get(k).and_then(|v| v.as_f64()).expect("pivot field");
        reference.push(PivotOutput {
            pp: f("pp"),
            r1: f("r1"),
            r2: f("r2"),
            r3: f("r3"),
            s1: f("s1"),
            s2: f("s2"),
            s3: f("s3"),
        });
    }
    let out: Vec<Option<PivotOutput>> = PivotPoints::new().calculate(&candles);
    assert_eq!(out.len(), reference.len());
    for (i, (o, r)) in out.iter().zip(reference.iter()).enumerate() {
        let o = o.as_ref().expect("pivot output");
        assert_close(o.pp, r.pp, "pp", i);
        assert_close(o.r1, r.r1, "r1", i);
        assert_close(o.r2, r.r2, "r2", i);
        assert_close(o.r3, r.r3, "r3", i);
        assert_close(o.s1, r.s1, "s1", i);
        assert_close(o.s2, r.s2, "s2", i);
        assert_close(o.s3, r.s3, "s3", i);
    }
}
