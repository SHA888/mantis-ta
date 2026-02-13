use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Simple Moving Average over closing prices.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, SMA};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [1.0, 2.0, 3.0, 4.0]
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
/// let values = SMA::new(3).calculate(&candles);
/// assert_eq!(values[0], None);
/// assert_eq!(values[1], None);
/// assert_eq!(values[2], Some(2.0));
/// assert_eq!(values[3], Some(3.0));
/// ```
#[derive(Debug, Clone)]
pub struct SMA {
    period: usize,
    sum: f64,
    window: RingBuf<f64>,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            sum: 0.0,
            window: RingBuf::new(period, 0.0),
        }
    }

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        let overwritten = self.window.push(value).unwrap_or(0.0);
        self.sum += value - overwritten;

        if self.window.len() >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

impl Indicator for SMA {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.close)
    }

    fn reset(&mut self) {
        self.sum = 0.0;
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
    fn computes_sma_after_warmup() {
        let mut sma = SMA::new(3);
        let candles = [1.0, 2.0, 3.0, 4.0]
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
            outputs.push(sma.next(c));
        }

        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        assert_eq!(outputs[2], Some(2.0));
        assert_eq!(outputs[3], Some(3.0));
    }
}
