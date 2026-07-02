use crate::indicators::Indicator;
use crate::types::Candle;

/// Wilder's Parabolic SAR (Stop And Reverse) trend-following indicator.
///
/// Tracks a trailing stop level that accelerates toward price as a trend
/// persists, and flips to the opposite side of price when the trend
/// reverses. `af_start` is the initial acceleration factor, `af_step` is
/// how much it grows each time a new extreme is reached, and `af_max`
/// caps it.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, ParabolicSar};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = (0..30)
///     .map(|i| {
///         let price = 100.0 + i as f64;
///         Candle {
///             timestamp: i as i64,
///             open: price,
///             high: price + 1.0,
///             low: price - 1.0,
///             close: price,
///             volume: 0.0,
///         }
///     })
///     .collect();
///
/// let out = ParabolicSar::new(0.02, 0.02, 0.2).calculate(&candles);
/// // Warmup period = 2: index 0 is None, index 1 is the first Some.
/// assert!(out[0].is_none());
/// assert!(out[1].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ParabolicSar {
    af_start: f64,
    af_step: f64,
    af_max: f64,
    bar_count: usize,
    first_high: f64,
    first_low: f64,
    first_close: f64,
    is_uptrend: bool,
    sar: f64,
    ep: f64,
    af: f64,
    prev1_high: f64,
    prev1_low: f64,
    prev2_high: f64,
    prev2_low: f64,
}

impl ParabolicSar {
    pub fn new(af_start: f64, af_step: f64, af_max: f64) -> Self {
        assert!(af_start > 0.0, "af_start must be > 0");
        assert!(af_step > 0.0, "af_step must be > 0");
        assert!(af_max >= af_start, "af_max must be >= af_start");
        Self {
            af_start,
            af_step,
            af_max,
            bar_count: 0,
            first_high: 0.0,
            first_low: 0.0,
            first_close: 0.0,
            is_uptrend: true,
            sar: 0.0,
            ep: 0.0,
            af: af_start,
            prev1_high: 0.0,
            prev1_low: 0.0,
            prev2_high: 0.0,
            prev2_low: 0.0,
        }
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        self.bar_count += 1;

        if self.bar_count == 1 {
            self.first_high = high;
            self.first_low = low;
            self.first_close = close;
            return None;
        }

        if self.bar_count == 2 {
            self.is_uptrend = close >= self.first_close;
            if self.is_uptrend {
                self.sar = self.first_low;
                self.ep = high.max(self.first_high);
            } else {
                self.sar = self.first_high;
                self.ep = low.min(self.first_low);
            }
            self.af = self.af_start;
            // The first recurrence step clamps only against this bar (not the
            // seed bar), matching TA-Lib's bootstrap: prev2 == prev1 here so
            // the two-bar clamp below degenerates to a one-bar clamp.
            self.prev2_high = high;
            self.prev2_low = low;
            self.prev1_high = high;
            self.prev1_low = low;
            return Some(self.sar);
        }

        let mut new_sar = self.sar + self.af * (self.ep - self.sar);
        if self.is_uptrend {
            new_sar = new_sar.min(self.prev1_low).min(self.prev2_low);
        } else {
            new_sar = new_sar.max(self.prev1_high).max(self.prev2_high);
        }

        if self.is_uptrend && low < new_sar {
            new_sar = self.ep.max(high).max(self.prev1_high);
            self.is_uptrend = false;
            self.ep = low;
            self.af = self.af_start;
        } else if !self.is_uptrend && high > new_sar {
            new_sar = self.ep.min(low).min(self.prev1_low);
            self.is_uptrend = true;
            self.ep = high;
            self.af = self.af_start;
        } else if self.is_uptrend {
            if high > self.ep {
                self.ep = high;
                self.af = (self.af + self.af_step).min(self.af_max);
            }
        } else if low < self.ep {
            self.ep = low;
            self.af = (self.af + self.af_step).min(self.af_max);
        }

        self.sar = new_sar;
        self.prev2_high = self.prev1_high;
        self.prev2_low = self.prev1_low;
        self.prev1_high = high;
        self.prev1_low = low;

        Some(self.sar)
    }
}

impl Indicator for ParabolicSar {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.high, candle.low, candle.close)
    }

    fn reset(&mut self) {
        self.bar_count = 0;
        self.first_high = 0.0;
        self.first_low = 0.0;
        self.first_close = 0.0;
        self.is_uptrend = true;
        self.sar = 0.0;
        self.ep = 0.0;
        self.af = self.af_start;
        self.prev1_high = 0.0;
        self.prev1_low = 0.0;
        self.prev2_high = 0.0;
        self.prev2_low = 0.0;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(i: i64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            timestamp: i,
            open: close,
            high,
            low,
            close,
            volume: 0.0,
        }
    }

    #[test]
    fn parabolic_sar_emits_after_warmup() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| {
                let price = 100.0 + i as f64;
                candle(i, price + 1.0, price - 1.0, price)
            })
            .collect();

        let out = ParabolicSar::new(0.02, 0.02, 0.2).calculate(&candles);
        assert!(out[0].is_none());
        assert!(out.iter().skip(1).all(|v| v.is_some()));
    }

    #[test]
    fn parabolic_sar_warmup_period_is_two() {
        let sar = ParabolicSar::new(0.02, 0.02, 0.2);
        assert_eq!(sar.warmup_period(), 2);
    }

    #[test]
    fn parabolic_sar_reset_restores_fresh_state() {
        let candles: Vec<Candle> = (0..5)
            .map(|i| {
                let price = 100.0 + i as f64;
                candle(i, price + 1.0, price - 1.0, price)
            })
            .collect();

        let mut sar = ParabolicSar::new(0.02, 0.02, 0.2);
        assert!(sar.next(&candles[0]).is_none());
        assert!(sar.next(&candles[1]).is_some());

        sar.reset();
        assert_eq!(sar.next(&candles[0]), None);
    }

    #[test]
    #[should_panic(expected = "af_start must be > 0")]
    fn parabolic_sar_rejects_non_positive_af_start() {
        ParabolicSar::new(0.0, 0.02, 0.2);
    }

    #[test]
    #[should_panic(expected = "af_max must be >= af_start")]
    fn parabolic_sar_rejects_af_max_below_af_start() {
        ParabolicSar::new(0.1, 0.02, 0.05);
    }
}
