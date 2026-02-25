use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Exponential Moving Average over closing prices.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, EMA};
/// use mantis_ta::types::Candle;
///
/// let prices = [1.0, 2.0, 3.0, 4.0];
/// let candles: Vec<Candle> = prices
///     .iter()
///     .enumerate()
///     .map(|(i, p)| Candle {
///         timestamp: i as i64,
///         open: *p,
///         high: *p,
///         low: *p,
///         close: *p,
///         volume: 0.0,
///     })
///     .collect();
///
/// let out = EMA::new(3).calculate(&candles);
/// // Warmup: first 2 bars None, third is seeded SMA
/// assert!(out[0].is_none());
/// assert!(out[1].is_none());
/// assert!(out[2].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: f64,
    warmup: RingBuf<f64>,
    ema: Option<f64>,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            warmup: RingBuf::new(period, 0.0),
            ema: None,
        }
    }

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        if let Some(prev) = self.ema {
            let next = (value - prev) * self.multiplier + prev;
            self.ema = Some(next);
            return self.ema;
        }

        // Warmup using simple average.
        self.warmup.push(value);
        if self.warmup.len() < self.period {
            return None;
        }
        let sum: f64 = self.warmup.iter().copied().sum();
        let sma = sum / self.period as f64;
        self.ema = Some(sma);
        self.ema
    }
}

impl Indicator for EMA {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.close)
    }

    fn reset(&mut self) {
        self.ema = None;
        self.warmup = RingBuf::new(self.period, 0.0);
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
    fn computes_ema_after_warmup() {
        let mut ema = EMA::new(3);
        let prices = [1.0, 2.0, 3.0, 4.0];
        let candles: Vec<Candle> = prices
            .iter()
            .map(|p| Candle {
                timestamp: 0,
                open: *p,
                high: *p,
                low: *p,
                close: *p,
                volume: 0.0,
            })
            .collect();

        let outputs: Vec<_> = candles.iter().map(|c| ema.next(c)).collect();
        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        assert!(outputs[2].is_some());
        assert!(outputs[3].is_some());
    }
}
