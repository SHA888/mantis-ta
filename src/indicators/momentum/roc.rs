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
/// assert_eq!(values[1], None);
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
            window: RingBuf::new(period + 1, 0.0),
        }
    }

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        self.window.push(value);

        if self.window.len() <= self.period {
            return None;
        }

        let current = value;
        let past = self.window.iter().nth(0).copied().unwrap_or(0.0);

        if past != 0.0 {
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
        self.window = RingBuf::new(self.period + 1, 0.0);
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
        assert_eq!(outputs[1], None);
        // ROC(2) at 104: ((104 - 100) / 100) * 100 = 4.0
        assert!(outputs[2].is_some());
        assert!((outputs[2].unwrap() - 4.0).abs() < 0.0001);
        // ROC(2) at 106: ((106 - 102) / 102) * 100 ≈ 3.922
        assert!(outputs[3].is_some());
        assert!((outputs[3].unwrap() - 3.922).abs() < 0.01);
    }
}
