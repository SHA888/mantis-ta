use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Commodity Channel Index measuring deviation from typical price.
///
/// CCI = (Typical Price - SMA of Typical Price) / (0.015 * Mean Deviation)
/// where Typical Price = (High + Low + Close) / 3
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, CCI};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [
///     (100.0, 102.0, 99.0),
///     (101.0, 103.0, 100.0),
///     (102.0, 104.0, 101.0),
///     (103.0, 105.0, 102.0),
/// ]
/// .iter()
/// .enumerate()
/// .map(|(i, (c, h, l))| Candle {
///     timestamp: i as i64,
///     open: *c,
///     high: *h,
///     low: *l,
///     close: *c,
///     volume: 0.0,
/// })
/// .collect();
///
/// let values = CCI::new(3).calculate(&candles);
/// assert_eq!(values[0], None);
/// assert_eq!(values[1], None);
/// assert!(values[2].is_some());
/// assert!(values[3].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct CCI {
    period: usize,
    tp_window: RingBuf<f64>,
}

impl CCI {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            tp_window: RingBuf::new(period, 0.0),
        }
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        let typical_price = (high + low + close) / 3.0;
        self.tp_window.push(typical_price);

        if self.tp_window.len() < self.period {
            return None;
        }

        let tp_values: Vec<f64> = self.tp_window.iter().copied().collect();
        let sma_tp = tp_values.iter().sum::<f64>() / self.period as f64;

        let mean_deviation = tp_values
            .iter()
            .map(|v| (v - sma_tp).abs())
            .sum::<f64>()
            / self.period as f64;

        if mean_deviation.abs() < 1e-10 {
            None
        } else {
            Some((typical_price - sma_tp) / (0.015 * mean_deviation))
        }
    }
}

impl Indicator for CCI {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.high, candle.low, candle.close)
    }

    fn reset(&mut self) {
        self.tp_window = RingBuf::new(self.period, 0.0);
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
    fn computes_cci_after_warmup() {
        let mut cci = CCI::new(3);
        let candles = vec![
            Candle {
                timestamp: 0,
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 100.0,
                volume: 0.0,
            },
            Candle {
                timestamp: 1,
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 101.0,
                volume: 0.0,
            },
            Candle {
                timestamp: 2,
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 102.0,
                volume: 0.0,
            },
            Candle {
                timestamp: 3,
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 103.0,
                volume: 0.0,
            },
        ];

        let mut outputs = Vec::new();
        for c in &candles {
            outputs.push(cci.next(c));
        }

        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        assert!(outputs[2].is_some());
        assert!(outputs[3].is_some());
    }
}
