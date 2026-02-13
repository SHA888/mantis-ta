use super::EMA;
use crate::indicators::Indicator;
use crate::types::{Candle, MacdOutput};

/// Moving Average Convergence Divergence over closing prices.
#[derive(Debug, Clone)]
pub struct MACD {
    fast: EMA,
    slow: EMA,
    signal: EMA,
    slow_period: usize,
    signal_period: usize,
}

impl MACD {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        assert!(fast > 0 && slow > 0 && signal > 0, "periods must be > 0");
        assert!(fast < slow, "fast period must be < slow period");
        Self {
            fast: EMA::new(fast),
            slow: EMA::new(slow),
            signal: EMA::new(signal),
            slow_period: slow,
            signal_period: signal,
        }
    }

    #[inline]
    fn macd_candle(value: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: value,
            high: value,
            low: value,
            close: value,
            volume: 0.0,
        }
    }
}

impl Indicator for MACD {
    type Output = MacdOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        let slow_val = self.slow.next(candle);
        let fast_val = self.fast.next(candle);

        let macd_line = match (fast_val, slow_val) {
            (Some(f), Some(s)) => f - s,
            _ => return None,
        };

        let macd_candle = Self::macd_candle(macd_line);
        let signal_line = match self.signal.next(&macd_candle) {
            Some(v) => v,
            None => return None,
        };

        let histogram = macd_line - signal_line;
        Some(MacdOutput {
            macd_line,
            signal_line,
            histogram,
        })
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.signal.reset();
    }

    fn warmup_period(&self) -> usize {
        // Need slow EMA warmup, then signal warmup on MACD line.
        self.slow_period + self.signal_period - 1
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macd_emits_after_warmup() {
        let mut macd = MACD::new(2, 4, 2);
        let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
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

        let outputs: Vec<_> = candles.iter().map(|c| macd.next(c)).collect();
        assert!(outputs
            .iter()
            .take(macd.warmup_period() - 1)
            .all(|o| o.is_none()));
        assert!(outputs
            .iter()
            .skip(macd.warmup_period() - 1)
            .any(|o| o.is_some()));
    }
}
