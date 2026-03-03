use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Rate of Change over closing prices.
///
/// Measures the percentage change in price over a specified period.
/// ROC = ((Close - Close[n periods ago]) / Close[n periods ago]) * 100
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, ROC};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [100.0, 102.0, 104.0, 106.0]
///     .iter()
///     .enumerate()
///     .map(|(i, c)| Candle {
///         timestamp: i as i64,
///         open: *c,
///         high: *c,
///         low: *c,
///         close: *c,
///         volume: 0.0,
///     })
///     .collect();
///
/// let values = ROC::new(2).calculate(&candles);
/// assert_eq!(values[0], None);
/// assert!(values[1].is_some());
/// assert!(values[2].is_some());
/// assert!(values[3].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ROC {
    period: usize,
    window: RingBuf<f64>,
}

impl ROC {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            window: RingBuf::new(period, 0.0),
        }
    }

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        self.window.push(value);

        if self.window.len() < self.period {
            return None;
        }

        let current = value;
        let past = self.window.iter().next().copied().unwrap_or(0.0);

        if past.abs() > 1e-10 {
            Some(((current - past) / past) * 100.0)
        } else {
            None
        }
    }
}

impl Indicator for ROC {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.close)
    }

    fn reset(&mut self) {
        self.window = RingBuf::new(self.period, 0.0);
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

    #[test]
    fn computes_roc_after_warmup() {
        let mut roc = ROC::new(2);
        let candles = [100.0, 102.0, 104.0, 106.0]
            .iter()
            .map(|c| Candle {
                timestamp: 0,
                open: *c,
                high: *c,
                low: *c,
                close: *c,
                volume: 0.0,
            })
            .collect::<Vec<_>>();

        let mut outputs = Vec::new();
        for c in &candles {
            outputs.push(roc.next(c));
        }

        assert_eq!(outputs[0], None);
        // ROC(2) at 102: ((102 - 100) / 100) * 100 = 2.0
        assert!(outputs[1].is_some());
        assert!((outputs[1].unwrap() - 2.0).abs() < 0.0001);
        // ROC(2) at 104: ((104 - 102) / 102) * 100 ≈ 1.961
        assert!(outputs[2].is_some());
        assert!((outputs[2].unwrap() - 1.961).abs() < 0.01);
        // ROC(2) at 106: ((106 - 104) / 104) * 100 ≈ 1.923
        assert!(outputs[3].is_some());
        assert!((outputs[3].unwrap() - 1.923).abs() < 0.01);
    }

    #[test]
    fn roc_reset_clears_state() {
        let mut roc = ROC::new(2);
        let candle = Candle {
            timestamp: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 0.0,
        };

        roc.next(&candle);
        roc.next(&candle);
        assert!(roc.next(&candle).is_some());

        roc.reset();
        assert_eq!(roc.next(&candle), None);
    }

    #[test]
    fn roc_with_negative_change() {
        let mut roc = ROC::new(2);
        let candles = [100.0, 95.0, 90.0]
            .iter()
            .map(|c| Candle {
                timestamp: 0,
                open: *c,
                high: *c,
                low: *c,
                close: *c,
                volume: 0.0,
            })
            .collect::<Vec<_>>();

        let outputs: Vec<_> = candles.iter().map(|c| roc.next(c)).collect();
        assert_eq!(outputs[0], None);
        // ROC(2) at 95: ((95 - 100) / 100) * 100 = -5.0
        assert!(outputs[1].is_some());
        assert!((outputs[1].unwrap() - (-5.0)).abs() < 0.0001);
        // ROC(2) at 90: ((90 - 95) / 95) * 100 ≈ -5.263
        assert!(outputs[2].is_some());
        assert!((outputs[2].unwrap() - (-5.263)).abs() < 0.01);
    }

    #[test]
    fn roc_warmup_period() {
        let roc = ROC::new(5);
        assert_eq!(roc.warmup_period(), 5);
    }
}
