use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Volume Weighted Average Price over a rolling window.
///
/// Typical Price = (High + Low + Close) / 3
/// VWAP = sum(Typical Price * Volume) / sum(Volume) over the last `period` bars.
///
/// TA-Lib has no native VWAP function; VWAP is conventionally a session-anchored
/// cumulative average, but `Candle` carries no session boundary. This
/// implementation instead uses a fixed rolling window (matching the convention
/// of this crate's other windowed indicators, e.g. `ATR`, `VolumeSMA`), so it
/// re-centers continuously rather than resetting at a session start.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, VWAP};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [
///     (11.0, 9.0, 10.0, 100.0),
///     (12.0, 10.0, 11.0, 200.0),
///     (13.0, 11.0, 12.0, 300.0),
/// ]
/// .iter()
/// .enumerate()
/// .map(|(i, (h, l, c, v))| Candle {
///     timestamp: i as i64,
///     open: *c,
///     high: *h,
///     low: *l,
///     close: *c,
///     volume: *v,
/// })
/// .collect();
///
/// let out = VWAP::new(3).calculate(&candles);
/// assert!(out.iter().take(2).all(|v| v.is_none()));
/// // Typical prices: 10, 11, 12; volumes: 100, 200, 300.
/// let expected = (10.0 * 100.0 + 11.0 * 200.0 + 12.0 * 300.0) / (100.0 + 200.0 + 300.0);
/// assert!((out[2].unwrap() - expected).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct VWAP {
    period: usize,
    window: RingBuf<(f64, f64)>,
    sum_tp_vol: f64,
    sum_vol: f64,
}

impl VWAP {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            window: RingBuf::new(period, (0.0, 0.0)),
            sum_tp_vol: 0.0,
            sum_vol: 0.0,
        }
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64, volume: f64) -> Option<f64> {
        let typical_price = (high + low + close) / 3.0;
        let tp_vol = typical_price * volume;

        if let Some((old_tp_vol, old_vol)) = self.window.push((tp_vol, volume)) {
            self.sum_tp_vol -= old_tp_vol;
            self.sum_vol -= old_vol;
        }
        self.sum_tp_vol += tp_vol;
        self.sum_vol += volume;

        if self.window.len() < self.period {
            None
        } else if self.sum_vol == 0.0 {
            Some(typical_price)
        } else {
            Some(self.sum_tp_vol / self.sum_vol)
        }
    }
}

impl Indicator for VWAP {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.high, candle.low, candle.close, candle.volume)
    }

    fn reset(&mut self) {
        self.window = RingBuf::new(self.period, (0.0, 0.0));
        self.sum_tp_vol = 0.0;
        self.sum_vol = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(h: f64, l: f64, c: f64, v: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: c,
            high: h,
            low: l,
            close: c,
            volume: v,
        }
    }

    #[test]
    fn vwap_warmup_period_is_period() {
        let vwap = VWAP::new(14);
        assert_eq!(vwap.warmup_period(), 14);
    }

    #[test]
    fn vwap_emits_none_until_warmup() {
        let mut vwap = VWAP::new(3);
        let candles = vec![
            candle(11.0, 9.0, 10.0, 100.0),
            candle(12.0, 10.0, 11.0, 110.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| vwap.next(c)).collect();
        assert!(outputs.iter().all(|v| v.is_none()));
    }

    #[test]
    fn vwap_matches_manual_calculation() {
        let mut vwap = VWAP::new(3);
        let candles = vec![
            candle(11.0, 9.0, 10.0, 100.0),
            candle(12.0, 10.0, 11.0, 200.0),
            candle(13.0, 11.0, 12.0, 300.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| vwap.next(c)).collect();

        let expected = (10.0 * 100.0 + 11.0 * 200.0 + 12.0 * 300.0) / (100.0 + 200.0 + 300.0);
        assert!((outputs[2].unwrap() - expected).abs() < 1e-9);
    }

    #[test]
    fn vwap_flat_prices_equals_typical_price() {
        let mut vwap = VWAP::new(3);
        let candles = vec![candle(51.0, 49.0, 50.0, 100.0); 3];
        let outputs: Vec<_> = candles.iter().map(|c| vwap.next(c)).collect();
        assert!((outputs[2].unwrap() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn vwap_zero_volume_window_falls_back_to_typical_price() {
        let mut vwap = VWAP::new(2);
        let candles = vec![candle(11.0, 9.0, 10.0, 0.0), candle(21.0, 19.0, 20.0, 0.0)];
        let outputs: Vec<_> = candles.iter().map(|c| vwap.next(c)).collect();
        assert!((outputs[1].unwrap() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn vwap_bounded_by_window_extremes() {
        // VWAP is a weighted average, so it must sit within [min typical price, max typical price]
        // of the window, regardless of volume weighting.
        let mut vwap = VWAP::new(4);
        let candles = vec![
            candle(11.0, 9.0, 10.0, 50.0),
            candle(21.0, 19.0, 20.0, 500.0),
            candle(16.0, 14.0, 15.0, 10.0),
            candle(31.0, 29.0, 30.0, 200.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| vwap.next(c)).collect();
        let v = outputs[3].unwrap();
        assert!((10.0..=30.0).contains(&v));
    }

    #[test]
    fn vwap_streaming_matches_batch() {
        let candles: Vec<Candle> = (0..15)
            .map(|i| {
                candle(
                    101.0 + i as f64,
                    99.0 + i as f64,
                    100.0 + i as f64,
                    1_000.0 + i as f64 * 10.0,
                )
            })
            .collect();

        let batch = VWAP::new(5).calculate(&candles);

        let mut streamed_vwap = VWAP::new(5);
        let streamed: Vec<_> = candles.iter().map(|c| streamed_vwap.next(c)).collect();

        assert_eq!(streamed, batch);
    }

    #[test]
    fn vwap_reset_clears_state() {
        let mut vwap = VWAP::new(4);
        let candles: Vec<Candle> = (0..6)
            .map(|i| candle(101.0 + i as f64, 99.0 + i as f64, 100.0 + i as f64, 1_000.0))
            .collect();
        for c in &candles {
            vwap.next(c);
        }
        vwap.reset();

        let mut fresh = VWAP::new(4);
        for c in &candles {
            assert_eq!(vwap.next(c), fresh.next(c));
        }
    }

    #[test]
    #[should_panic(expected = "period must be > 0")]
    fn vwap_rejects_zero_period() {
        VWAP::new(0);
    }
}
