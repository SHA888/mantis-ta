use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Standard Deviation of closing prices.
///
/// Measures the volatility of price movements over a specified period.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, StdDev};
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
/// let values = StdDev::new(3).calculate(&candles);
/// assert_eq!(values[0], None);
/// assert_eq!(values[1], None);
/// assert!(values[2].is_some());
/// assert!(values[3].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct StdDev {
    period: usize,
    window: RingBuf<f64>,
}

impl StdDev {
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

        let mean = self.window.iter().sum::<f64>() / self.period as f64;
        let variance = self
            .window
            .iter()
            .map(|v| {
                let diff = v - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.period as f64;

        Some(variance.sqrt())
    }
}

impl Indicator for StdDev {
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
    fn computes_stddev_after_warmup() {
        let mut stddev = StdDev::new(3);
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
            outputs.push(stddev.next(c));
        }

        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        // StdDev([1,2,3]): mean=2, variance=((1-2)^2 + (2-2)^2 + (3-2)^2)/3 = 2/3, stddev=sqrt(2/3)≈0.8165
        assert!(outputs[2].is_some());
        assert!((outputs[2].unwrap() - 0.8165).abs() < 0.001);
        assert!(outputs[3].is_some());
    }

    #[test]
    fn stddev_reset_clears_state() {
        let mut stddev = StdDev::new(3);
        let candle = Candle {
            timestamp: 0,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 0.0,
        };

        stddev.next(&candle);
        stddev.next(&candle);
        stddev.next(&candle);
        assert!(stddev.next(&candle).is_some());

        stddev.reset();
        assert_eq!(stddev.next(&candle), None);
    }

    #[test]
    fn stddev_with_constant_values() {
        let mut stddev = StdDev::new(3);
        let candles = [5.0, 5.0, 5.0]
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

        let outputs: Vec<_> = candles.iter().map(|c| stddev.next(c)).collect();
        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        assert_eq!(outputs[2], Some(0.0));
    }

    #[test]
    fn stddev_warmup_period() {
        let stddev = StdDev::new(5);
        assert_eq!(stddev.warmup_period(), 5);
    }
}
