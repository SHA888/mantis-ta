use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Weighted Moving Average over closing prices.
///
/// Assigns linearly increasing weights to recent prices, giving more importance
/// to recent data compared to SMA.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, WMA};
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
/// let values = WMA::new(3).calculate(&candles);
/// assert_eq!(values[0], None);
/// assert_eq!(values[1], None);
/// assert!(values[2].is_some());
/// assert!(values[3].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct WMA {
    period: usize,
    window: RingBuf<f64>,
    divisor: f64,
}

impl WMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        let divisor = (period * (period + 1) / 2) as f64;
        Self {
            period,
            window: RingBuf::new(period, 0.0),
            divisor,
        }
    }

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        self.window.push(value);

        if self.window.len() < self.period {
            return None;
        }

        let weighted_sum: f64 = self
            .window
            .iter()
            .enumerate()
            .map(|(i, &v)| v * ((i + 1) as f64))
            .sum();

        Some(weighted_sum / self.divisor)
    }
}

impl Indicator for WMA {
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
    fn computes_wma_after_warmup() {
        let mut wma = WMA::new(3);
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
            outputs.push(wma.next(c));
        }

        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        // WMA(3) at [1,2,3]: (1*1 + 2*2 + 3*3) / 6 = 14/6 = 2.333...
        assert!(outputs[2].is_some());
        let wma_val = outputs[2].unwrap();
        assert!((wma_val - 2.333333).abs() < 0.0001);
        assert!(outputs[3].is_some());
    }

    #[test]
    fn wma_reset_clears_state() {
        let mut wma = WMA::new(3);
        let candle = Candle {
            timestamp: 0,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 0.0,
        };

        wma.next(&candle);
        wma.next(&candle);
        wma.next(&candle);
        assert!(wma.next(&candle).is_some());

        wma.reset();
        assert_eq!(wma.next(&candle), None);
    }

    #[test]
    fn wma_with_constant_values() {
        let mut wma = WMA::new(2);
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

        let outputs: Vec<_> = candles.iter().map(|c| wma.next(c)).collect();
        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], Some(5.0));
        assert_eq!(outputs[2], Some(5.0));
    }

    #[test]
    fn wma_warmup_period() {
        let wma = WMA::new(5);
        assert_eq!(wma.warmup_period(), 5);
    }
}
