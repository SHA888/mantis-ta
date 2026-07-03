use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Money Flow Index — a volume-weighted RSI.
///
/// Typical Price = (High + Low + Close) / 3
/// Raw Money Flow = Typical Price * Volume
/// Each bar's raw money flow is classified positive if typical price rose
/// versus the prior bar, negative if it fell, and excluded from both sums
/// if typical price is unchanged. MFI sums positive and negative money flow
/// over a rolling `period`-bar window:
///
/// Money Ratio = (sum of positive money flow) / (sum of negative money flow)
/// MFI = 100 - 100 / (1 + Money Ratio)
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, MFI};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [
///     (10.0, 11.0, 9.0, 100.0),
///     (11.0, 12.0, 10.0, 110.0),
///     (12.0, 13.0, 11.0, 120.0),
///     (11.0, 12.0, 10.0, 90.0),
///     (10.0, 11.0, 9.0, 95.0),
/// ]
/// .iter()
/// .enumerate()
/// .map(|(i, (o, h, l, v))| Candle {
///     timestamp: i as i64,
///     open: *o,
///     high: *h,
///     low: *l,
///     close: *o,
///     volume: *v,
/// })
/// .collect();
///
/// let values = MFI::new(3).calculate(&candles);
/// // Warmup: period + 1 bars before first value
/// assert!(values.iter().take(3).all(|v| v.is_none()));
/// assert!(values[3].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MFI {
    period: usize,
    prev_tp: Option<f64>,
    flow_window: RingBuf<(f64, f64)>,
    pos_sum: f64,
    neg_sum: f64,
}

impl MFI {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            prev_tp: None,
            flow_window: RingBuf::new(period, (0.0, 0.0)),
            pos_sum: 0.0,
            neg_sum: 0.0,
        }
    }

    #[inline]
    fn compute_mfi(pos_sum: f64, neg_sum: f64) -> f64 {
        if neg_sum == 0.0 {
            return 100.0;
        }
        let money_ratio = pos_sum / neg_sum;
        100.0 - 100.0 / (1.0 + money_ratio)
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64, volume: f64) -> Option<f64> {
        let typical_price = (high + low + close) / 3.0;
        let Some(prev) = self.prev_tp else {
            self.prev_tp = Some(typical_price);
            return None;
        };
        self.prev_tp = Some(typical_price);

        let raw_flow = typical_price * volume;
        let (pos, neg) = match typical_price.partial_cmp(&prev) {
            Some(std::cmp::Ordering::Greater) => (raw_flow, 0.0),
            Some(std::cmp::Ordering::Less) => (0.0, raw_flow),
            _ => (0.0, 0.0),
        };

        if let Some((old_pos, old_neg)) = self.flow_window.push((pos, neg)) {
            self.pos_sum -= old_pos;
            self.neg_sum -= old_neg;
        }
        self.pos_sum += pos;
        self.neg_sum += neg;

        if self.flow_window.len() < self.period {
            None
        } else {
            Some(Self::compute_mfi(self.pos_sum, self.neg_sum))
        }
    }
}

impl Indicator for MFI {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.high, candle.low, candle.close, candle.volume)
    }

    fn reset(&mut self) {
        self.prev_tp = None;
        self.flow_window = RingBuf::new(self.period, (0.0, 0.0));
        self.pos_sum = 0.0;
        self.neg_sum = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
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
    fn mfi_warmup_period_is_period_plus_one() {
        let mfi = MFI::new(14);
        assert_eq!(mfi.warmup_period(), 15);
    }

    #[test]
    fn mfi_emits_none_until_warmup() {
        let mut mfi = MFI::new(3);
        let candles = vec![
            candle(11.0, 9.0, 10.0, 100.0),
            candle(12.0, 10.0, 11.0, 110.0),
            candle(13.0, 11.0, 12.0, 120.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| mfi.next(c)).collect();
        assert!(outputs.iter().all(|v| v.is_none()));
    }

    #[test]
    fn mfi_all_rising_typical_price_saturates_at_100() {
        let mut mfi = MFI::new(3);
        let candles = vec![
            candle(11.0, 9.0, 10.0, 100.0),
            candle(12.0, 10.0, 11.0, 110.0),
            candle(13.0, 11.0, 12.0, 120.0),
            candle(14.0, 12.0, 13.0, 130.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| mfi.next(c)).collect();
        assert!(outputs[3].is_some());
        assert!((outputs[3].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn mfi_all_falling_typical_price_saturates_at_0() {
        let mut mfi = MFI::new(3);
        let candles = vec![
            candle(14.0, 12.0, 13.0, 130.0),
            candle(13.0, 11.0, 12.0, 120.0),
            candle(12.0, 10.0, 11.0, 110.0),
            candle(11.0, 9.0, 10.0, 100.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| mfi.next(c)).collect();
        assert!(outputs[3].is_some());
        assert!(outputs[3].unwrap().abs() < 1e-9);
    }

    #[test]
    fn mfi_flat_typical_price_excluded_from_both_sums() {
        // Bar 1->2 has an unchanged typical price; it must contribute to
        // neither the positive nor negative money flow sum.
        let mut mfi = MFI::new(3);
        let candles = vec![
            candle(11.0, 9.0, 9.5, 100.0),
            candle(12.0, 10.0, 10.5, 110.0),
            candle(12.0, 10.0, 10.5, 120.0), // flat vs bar 2
            candle(13.0, 11.0, 11.5, 130.0),
        ];
        let outputs: Vec<_> = candles.iter().map(|c| mfi.next(c)).collect();
        // window covers diffs at bars 2,3,4: positive, flat(excluded), positive
        // => neg_sum stays 0 => MFI saturates at 100 despite the flat bar.
        assert!(outputs[3].is_some());
        assert!((outputs[3].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn mfi_reset_clears_state() {
        let mut mfi = MFI::new(3);
        let candles = vec![
            candle(11.0, 9.0, 10.0, 100.0),
            candle(12.0, 10.0, 11.0, 110.0),
            candle(13.0, 11.0, 12.0, 120.0),
            candle(14.0, 12.0, 13.0, 130.0),
        ];
        for c in &candles {
            mfi.next(c);
        }
        mfi.reset();
        assert!(mfi.next(&candles[0]).is_none());
        assert_eq!(mfi.pos_sum, 0.0);
        assert_eq!(mfi.neg_sum, 0.0);
    }

    #[test]
    fn mfi_streaming_matches_batch() {
        let candles = vec![
            candle(11.0, 9.0, 10.0, 100.0),
            candle(12.0, 10.0, 11.0, 110.0),
            candle(13.0, 11.0, 12.0, 120.0),
            candle(14.0, 12.0, 13.0, 130.0),
            candle(12.0, 10.0, 11.0, 90.0),
            candle(11.0, 9.0, 10.0, 95.0),
        ];
        let batch = MFI::new(3).calculate(&candles);
        let mut streaming_out = Vec::new();
        let mut mfi = MFI::new(3);
        for c in &candles {
            streaming_out.push(mfi.next(c));
        }
        assert_eq!(batch, streaming_out);
    }
}
