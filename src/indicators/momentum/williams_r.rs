use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Williams %R (Williams Percent Range) momentum oscillator.
///
/// Measures the level of the close relative to the high-low range over a period.
/// Williams %R = ((Highest High - Close) / (Highest High - Lowest Low)) * -100
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, WilliamsR};
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
/// let values = WilliamsR::new(3).calculate(&candles);
/// assert_eq!(values[0], None);
/// assert_eq!(values[1], None);
/// assert!(values[2].is_some());
/// assert!(values[3].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct WilliamsR {
    period: usize,
    highs: RingBuf<f64>,
    lows: RingBuf<f64>,
}

impl WilliamsR {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            highs: RingBuf::new(period, 0.0),
            lows: RingBuf::new(period, 0.0),
        }
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        self.highs.push(high);
        self.lows.push(low);

        if self.highs.len() < self.period {
            return None;
        }

        let highest_high = self.highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lowest_low = self.lows.iter().copied().fold(f64::INFINITY, f64::min);

        let range = highest_high - lowest_low;
        if range.abs() < 1e-10 {
            Some(0.0)
        } else {
            Some(((highest_high - close) / range) * -100.0)
        }
    }
}

impl Indicator for WilliamsR {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.high, candle.low, candle.close)
    }

    fn reset(&mut self) {
        self.highs = RingBuf::new(self.period, 0.0);
        self.lows = RingBuf::new(self.period, 0.0);
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
    fn computes_williams_r_after_warmup() {
        let mut wr = WilliamsR::new(3);
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
            outputs.push(wr.next(c));
        }

        assert_eq!(outputs[0], None);
        assert_eq!(outputs[1], None);
        assert!(outputs[2].is_some());
        assert!(outputs[3].is_some());
        // At bar 2: highest=104, lowest=99, close=102
        // %R = ((104 - 102) / (104 - 99)) * -100 = (2/5) * -100 = -40
        assert!((outputs[2].unwrap() - (-40.0)).abs() < 0.01);
    }

    #[test]
    fn williams_r_reset_clears_state() {
        let mut wr = WilliamsR::new(3);
        let candle = Candle {
            timestamp: 0,
            open: 100.0,
            high: 102.0,
            low: 99.0,
            close: 100.0,
            volume: 0.0,
        };

        wr.next(&candle);
        wr.next(&candle);
        wr.next(&candle);
        assert!(wr.next(&candle).is_some());

        wr.reset();
        assert_eq!(wr.next(&candle), None);
    }

    #[test]
    fn williams_r_at_high() {
        let mut wr = WilliamsR::new(2);
        let candles = vec![
            Candle {
                timestamp: 0,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 0.0,
            },
            Candle {
                timestamp: 1,
                open: 105.0,
                high: 105.0,
                low: 100.0,
                close: 105.0,
                volume: 0.0,
            },
        ];

        let outputs: Vec<_> = candles.iter().map(|c| wr.next(c)).collect();
        assert_eq!(outputs[0], None);
        // At bar 1: highest=105, lowest=100, close=105
        // %R = ((105 - 105) / (105 - 100)) * -100 = 0
        assert_eq!(outputs[1], Some(0.0));
    }

    #[test]
    fn williams_r_warmup_period() {
        let wr = WilliamsR::new(5);
        assert_eq!(wr.warmup_period(), 5);
    }
}
