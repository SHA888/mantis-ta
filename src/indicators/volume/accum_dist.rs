use crate::indicators::Indicator;
use crate::types::Candle;

/// Accumulation/Distribution Line.
///
/// A cumulative volume-flow indicator. Each bar's volume is weighted by a
/// Close Location Value (money flow multiplier):
///
/// ```text
/// MFM = ((Close - Low) - (High - Close)) / (High - Low)
/// AD  = AD_prev + MFM * Volume
/// ```
///
/// `MFM` ranges over `[-1, 1]` and measures where the close settled within
/// the bar's high/low range (`+1` = close at the high, `-1` = close at the
/// low). When `High == Low` the multiplier is defined as `0` (no
/// accumulation or distribution on a zero-range bar), matching TA-Lib's
/// `AD` function.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{AccumDist, Indicator};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [
///     (12.0, 8.0, 10.0, 100.0),
///     (13.0, 9.0, 12.0, 200.0),
///     (11.0, 7.0, 8.0, 150.0),
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
/// let out = AccumDist::new().calculate(&candles);
/// assert_eq!(out[0], Some(0.0)); // ((10-8)-(12-10))/4 * 100 = 0
/// assert_eq!(out[1], Some(100.0)); // 0 + ((12-9)-(13-12))/4 * 200 = 100
/// assert_eq!(out[2], Some(25.0)); // 100 + ((8-7)-(11-8))/4 * 150 = 25
/// ```
#[derive(Debug, Clone)]
pub struct AccumDist {
    current: f64,
}

impl AccumDist {
    pub fn new() -> Self {
        Self { current: 0.0 }
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64, volume: f64) -> f64 {
        let range = high - low;
        let money_flow_multiplier = if range == 0.0 {
            0.0
        } else {
            ((close - low) - (high - close)) / range
        };
        self.current += money_flow_multiplier * volume;
        self.current
    }
}

impl Default for AccumDist {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for AccumDist {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        Some(self.update(candle.high, candle.low, candle.close, candle.volume))
    }

    fn reset(&mut self) {
        self.current = 0.0;
    }

    fn warmup_period(&self) -> usize {
        0
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
    fn accum_dist_warmup_period_is_zero() {
        assert_eq!(AccumDist::new().warmup_period(), 0);
    }

    #[test]
    fn accum_dist_matches_manual_calculation() {
        let mut ad = AccumDist::new();
        let candles = vec![
            candle(12.0, 8.0, 10.0, 100.0),
            candle(13.0, 9.0, 12.0, 200.0),
            candle(11.0, 7.0, 8.0, 150.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| ad.next(c)).collect();

        assert!((outputs[0].unwrap() - 0.0).abs() < 1e-9);
        assert!((outputs[1].unwrap() - 100.0).abs() < 1e-9);
        assert!((outputs[2].unwrap() - 25.0).abs() < 1e-9);
    }

    #[test]
    fn accum_dist_zero_range_bar_does_not_panic_or_change_total() {
        let mut ad = AccumDist::new();
        ad.next(&candle(100.0, 100.0, 100.0, 10_000.0));
        assert_eq!(ad.next(&candle(100.0, 100.0, 100.0, 10_000.0)), Some(0.0));
    }

    #[test]
    fn accum_dist_close_at_high_adds_full_volume() {
        let mut ad = AccumDist::new();
        assert_eq!(ad.next(&candle(110.0, 90.0, 110.0, 500.0)), Some(500.0));
    }

    #[test]
    fn accum_dist_close_at_low_subtracts_full_volume() {
        let mut ad = AccumDist::new();
        assert_eq!(ad.next(&candle(110.0, 90.0, 90.0, 500.0)), Some(-500.0));
    }

    #[test]
    fn accum_dist_streaming_matches_batch() {
        let candles: Vec<Candle> = (0..15)
            .map(|i| {
                candle(
                    101.0 + i as f64,
                    99.0 - i as f64 * 0.5,
                    100.0 + i as f64 * 0.3,
                    1_000.0 + i as f64 * 10.0,
                )
            })
            .collect();

        let batch = AccumDist::new().calculate(&candles);

        let mut streamed_ad = AccumDist::new();
        let streamed: Vec<_> = candles.iter().map(|c| streamed_ad.next(c)).collect();

        assert_eq!(streamed, batch);
    }

    #[test]
    fn accum_dist_reset_clears_state() {
        let mut ad = AccumDist::new();
        let candles: Vec<Candle> = (0..6)
            .map(|i| candle(101.0 + i as f64, 99.0, 100.0 + i as f64 * 0.5, 1_000.0))
            .collect();
        for c in &candles {
            ad.next(c);
        }
        ad.reset();

        let mut fresh = AccumDist::new();
        for c in &candles {
            assert_eq!(ad.next(c), fresh.next(c));
        }
    }
}
