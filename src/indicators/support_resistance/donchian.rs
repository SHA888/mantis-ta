use crate::indicators::Indicator;
use crate::types::{Candle, DonchianOutput};
use crate::utils::ringbuf::RingBuf;

/// Donchian Channels — highest high / lowest low over a rolling window.
///
/// The upper band is the highest high and the lower band the lowest low
/// over the last `period` candles (inclusive of the current one); the
/// middle band is their midpoint. Commonly used for breakout detection
/// and trailing-stop placement.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{DonchianChannels, Indicator};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = (0..10)
///     .map(|i| {
///         let price = 100.0 + i as f64;
///         Candle {
///             timestamp: i as i64,
///             open: price,
///             high: price + 1.0,
///             low: price - 1.0,
///             close: price,
///             volume: 1_000.0,
///         }
///     })
///     .collect();
///
/// let out = DonchianChannels::new(5).calculate(&candles);
/// assert!(out.iter().take(4).all(|v| v.is_none()));
/// let d = out[4].unwrap();
/// assert!(d.upper > d.middle && d.middle > d.lower);
/// ```
#[derive(Debug, Clone)]
pub struct DonchianChannels {
    period: usize,
    window: RingBuf<(f64, f64)>,
}

impl DonchianChannels {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            window: RingBuf::new(period, (0.0, 0.0)),
        }
    }
}

impl Indicator for DonchianChannels {
    type Output = DonchianOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.window.push((candle.high, candle.low));
        if self.window.len() < self.period {
            return None;
        }

        let mut upper = f64::MIN;
        let mut lower = f64::MAX;
        for &(h, l) in self.window.iter() {
            if h > upper {
                upper = h;
            }
            if l < lower {
                lower = l;
            }
        }

        Some(DonchianOutput {
            upper,
            middle: (upper + lower) / 2.0,
            lower,
        })
    }

    fn reset(&mut self) {
        self.window = RingBuf::new(self.period, (0.0, 0.0));
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

    fn candle(high: f64, low: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: (high + low) / 2.0,
            high,
            low,
            close: (high + low) / 2.0,
            volume: 1_000.0,
        }
    }

    #[test]
    fn donchian_emits_after_warmup() {
        let mut dc = DonchianChannels::new(5);
        let candles: Vec<Candle> = (0..8)
            .map(|i| candle(100.0 + i as f64, 99.0 + i as f64))
            .collect();

        let outputs: Vec<_> = candles.iter().map(|c| dc.next(c)).collect();
        let wp = dc.warmup_period();
        assert!(outputs.iter().take(wp - 1).all(|o| o.is_none()));
        assert!(outputs[wp - 1].is_some());
    }

    #[test]
    fn donchian_tracks_highest_high_lowest_low() {
        let mut dc = DonchianChannels::new(3);
        let candles: Vec<Candle> = [(10.0, 5.0), (12.0, 6.0), (8.0, 3.0), (9.0, 4.0), (7.0, 2.0)]
            .iter()
            .map(|&(h, l)| candle(h, l))
            .collect();

        let outputs: Vec<_> = candles.iter().map(|c| dc.next(c)).collect();

        // Window [10,12,8] high=5,6,3 low -> upper=12, lower=3
        let out2 = outputs[2].unwrap();
        assert!((out2.upper - 12.0).abs() < 1e-9);
        assert!((out2.lower - 3.0).abs() < 1e-9);
        assert!((out2.middle - 7.5).abs() < 1e-9);

        // Window [12,8,9] high, [6,3,4] low -> upper=12, lower=3
        let out3 = outputs[3].unwrap();
        assert!((out3.upper - 12.0).abs() < 1e-9);
        assert!((out3.lower - 3.0).abs() < 1e-9);

        // Window [8,9,7] high, [3,4,2] low -> upper=9, lower=2
        let out4 = outputs[4].unwrap();
        assert!((out4.upper - 9.0).abs() < 1e-9);
        assert!((out4.lower - 2.0).abs() < 1e-9);
    }

    #[test]
    fn donchian_bands_bracket_middle() {
        let mut dc = DonchianChannels::new(4);
        let candles: Vec<Candle> = (0..8)
            .map(|i| candle(100.0 + i as f64, 99.0 - i as f64))
            .collect();

        for c in &candles {
            if let Some(out) = dc.next(c) {
                assert!(out.upper >= out.middle);
                assert!(out.middle >= out.lower);
                assert!((out.middle - (out.upper + out.lower) / 2.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn donchian_flat_prices_zero_width_band() {
        let mut dc = DonchianChannels::new(3);
        let candles: Vec<Candle> = (0..5).map(|_| candle(50.0, 50.0)).collect();

        for c in &candles {
            if let Some(out) = dc.next(c) {
                assert!((out.upper - 50.0).abs() < 1e-9);
                assert!((out.lower - 50.0).abs() < 1e-9);
                assert!((out.middle - 50.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn donchian_streaming_matches_batch() {
        let candles: Vec<Candle> = (0..15)
            .map(|i| candle(90.0 + i as f64 * 0.7, 89.0 + i as f64 * 0.7))
            .collect();

        let batch = DonchianChannels::new(5).calculate(&candles);

        let mut streamed_dc = DonchianChannels::new(5);
        let streamed: Vec<_> = candles.iter().map(|c| streamed_dc.next(c)).collect();

        assert_eq!(streamed, batch);
    }

    #[test]
    fn donchian_reset_clears_state() {
        let mut dc = DonchianChannels::new(4);
        let candles: Vec<Candle> = (0..6)
            .map(|i| candle(100.0 + i as f64, 99.0 + i as f64))
            .collect();
        for c in &candles {
            dc.next(c);
        }
        dc.reset();

        let mut fresh = DonchianChannels::new(4);
        for c in &candles {
            assert_eq!(dc.next(c), fresh.next(c));
        }
    }

    #[test]
    #[should_panic(expected = "period must be > 0")]
    fn donchian_rejects_zero_period() {
        DonchianChannels::new(0);
    }
}
