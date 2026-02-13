use crate::indicators::Indicator;
use crate::types::{Candle, StochasticOutput};
use crate::utils::ringbuf::RingBuf;

/// Stochastic Oscillator (%K and %D) over high/low/close.
#[derive(Debug, Clone)]
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
    window: RingBuf<(f64, f64)>,
    k_values: RingBuf<f64>,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        assert!(k_period > 0, "k_period must be > 0");
        assert!(d_period > 0, "d_period must be > 0");
        Self {
            k_period,
            d_period,
            window: RingBuf::new(k_period, (0.0, 0.0)),
            k_values: RingBuf::new(d_period, 0.0),
        }
    }

    fn range_high_low(&self) -> Option<(f64, f64)> {
        if self.window.len() < self.k_period {
            return None;
        }
        let mut highest = f64::MIN;
        let mut lowest = f64::MAX;
        for (high, low) in self.window.iter() {
            if *high > highest {
                highest = *high;
            }
            if *low < lowest {
                lowest = *low;
            }
        }
        Some((highest, lowest))
    }
}

impl Indicator for Stochastic {
    type Output = StochasticOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.window.push((candle.high, candle.low));

        let (highest, lowest) = self.range_high_low()?;

        let denom = highest - lowest;
        let k = if denom == 0.0 {
            50.0
        } else {
            ((candle.close - lowest) / denom) * 100.0
        };
        self.k_values.push(k);

        if self.k_values.len() < self.d_period {
            return None;
        }

        let sum_d: f64 = self.k_values.iter().copied().sum();
        let d = sum_d / self.d_period as f64;

        Some(StochasticOutput { k, d })
    }

    fn reset(&mut self) {
        self.window = RingBuf::new(self.k_period, (0.0, 0.0));
        self.k_values = RingBuf::new(self.d_period, 0.0);
    }

    fn warmup_period(&self) -> usize {
        self.k_period + self.d_period - 1
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stochastic_emits_after_warmup() {
        let mut stoch = Stochastic::new(3, 3);
        let candles: Vec<Candle> = [
            (1.0, 0.5, 0.8),
            (2.0, 0.5, 1.5),
            (3.0, 1.0, 2.5),
            (3.5, 1.5, 3.0),
            (4.0, 2.0, 3.5),
        ]
        .iter()
        .map(|(h, l, c)| Candle {
            timestamp: 0,
            open: *c,
            high: *h,
            low: *l,
            close: *c,
            volume: 0.0,
        })
        .collect();

        let outputs: Vec<_> = candles.iter().map(|c| stoch.next(c)).collect();
        assert!(outputs
            .iter()
            .take(stoch.warmup_period() - 1)
            .all(|o| o.is_none()));
        assert!(outputs
            .iter()
            .skip(stoch.warmup_period() - 1)
            .any(|o| o.is_some()));
    }
}
