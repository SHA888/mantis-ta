use crate::indicators::Indicator;
use crate::types::{Candle, IchimokuOutput};
use crate::utils::ringbuf::RingBuf;

/// Ichimoku Cloud (Ichimoku Kinko Hyo) — a multi-line trend and
/// support/resistance system built from rolling high/low midpoints.
///
/// Components (all computed from data up to and including the current bar):
/// - `tenkan_sen` (conversion line): midpoint of the `tenkan_period` high/low
/// - `kijun_sen` (base line): midpoint of the `kijun_period` high/low
/// - `senkou_span_a` (leading span A): midpoint of `tenkan_sen`/`kijun_sen`
/// - `senkou_span_b` (leading span B): midpoint of the `senkou_b_period` high/low
/// - `chikou_span` (lagging span): the current close
///
/// On a classic Ichimoku chart, Senkou Span A/B are plotted `kijun_period`
/// bars *ahead* and Chikou Span is plotted `kijun_period` bars *behind* —
/// that plotting displacement is a presentation concern, so this streaming
/// indicator returns the undisplaced values and leaves the offset to the
/// caller (chart renderer or strategy).
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Ichimoku, Indicator};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = (0..60)
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
/// // Standard 9/26/52 periods; full output needs 52 candles.
/// let out = Ichimoku::new(9, 26, 52).calculate(&candles);
/// assert!(out.iter().take(51).all(|v| v.is_none()));
/// assert!(out[51].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Ichimoku {
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    tenkan_window: RingBuf<(f64, f64)>,
    kijun_window: RingBuf<(f64, f64)>,
    senkou_b_window: RingBuf<(f64, f64)>,
}

impl Ichimoku {
    pub fn new(tenkan_period: usize, kijun_period: usize, senkou_b_period: usize) -> Self {
        assert!(
            tenkan_period > 0 && kijun_period > 0 && senkou_b_period > 0,
            "periods must be > 0"
        );
        Self {
            tenkan_period,
            kijun_period,
            senkou_b_period,
            tenkan_window: RingBuf::new(tenkan_period, (0.0, 0.0)),
            kijun_window: RingBuf::new(kijun_period, (0.0, 0.0)),
            senkou_b_window: RingBuf::new(senkou_b_period, (0.0, 0.0)),
        }
    }

    #[inline]
    fn midpoint(window: &RingBuf<(f64, f64)>, period: usize) -> Option<f64> {
        if window.len() < period {
            return None;
        }
        let mut highest = f64::MIN;
        let mut lowest = f64::MAX;
        for &(high, low) in window.iter() {
            if high > highest {
                highest = high;
            }
            if low < lowest {
                lowest = low;
            }
        }
        Some((highest + lowest) / 2.0)
    }
}

impl Indicator for Ichimoku {
    type Output = IchimokuOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.tenkan_window.push((candle.high, candle.low));
        self.kijun_window.push((candle.high, candle.low));
        self.senkou_b_window.push((candle.high, candle.low));

        let tenkan_sen = Self::midpoint(&self.tenkan_window, self.tenkan_period)?;
        let kijun_sen = Self::midpoint(&self.kijun_window, self.kijun_period)?;
        let senkou_span_b = Self::midpoint(&self.senkou_b_window, self.senkou_b_period)?;
        let senkou_span_a = (tenkan_sen + kijun_sen) / 2.0;

        Some(IchimokuOutput {
            tenkan_sen,
            kijun_sen,
            senkou_span_a,
            senkou_span_b,
            chikou_span: candle.close,
        })
    }

    fn reset(&mut self) {
        self.tenkan_window = RingBuf::new(self.tenkan_period, (0.0, 0.0));
        self.kijun_window = RingBuf::new(self.kijun_period, (0.0, 0.0));
        self.senkou_b_window = RingBuf::new(self.senkou_b_period, (0.0, 0.0));
    }

    fn warmup_period(&self) -> usize {
        self.tenkan_period
            .max(self.kijun_period)
            .max(self.senkou_b_period)
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candles(n: usize) -> Vec<Candle> {
        (0..n)
            .map(|i| {
                let price = 100.0 + i as f64;
                Candle {
                    timestamp: i as i64,
                    open: price,
                    high: price + 1.0,
                    low: price - 1.0,
                    close: price,
                    volume: 0.0,
                }
            })
            .collect()
    }

    #[test]
    fn ichimoku_emits_after_warmup() {
        let candles = make_candles(60);
        let ichimoku = Ichimoku::new(9, 26, 52);
        let out = ichimoku.calculate(&candles);

        assert!(out.iter().take(51).all(|v| v.is_none()));
        assert!(out[51].is_some());
    }

    #[test]
    fn ichimoku_warmup_period_is_max_of_periods() {
        assert_eq!(Ichimoku::new(9, 26, 52).warmup_period(), 52);
        assert_eq!(Ichimoku::new(9, 52, 26).warmup_period(), 52);
    }

    #[test]
    fn ichimoku_reset_restores_fresh_state() {
        let candles = make_candles(60);
        let mut ichimoku = Ichimoku::new(9, 26, 52);
        for candle in &candles {
            ichimoku.next(candle);
        }
        assert!(ichimoku.next(&candles[0]).is_some());

        ichimoku.reset();
        assert_eq!(ichimoku.next(&candles[0]), None);
    }

    #[test]
    #[should_panic(expected = "periods must be > 0")]
    fn ichimoku_rejects_zero_period() {
        Ichimoku::new(0, 26, 52);
    }
}
