use crate::indicators::Indicator;
use crate::types::{BollingerOutput, Candle};
use crate::utils::ringbuf::RingBuf;

/// Bollinger Bands over closing prices.
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_mult: f64,
    window: RingBuf<f64>,
    sum: f64,
    sum_sq: f64,
}

impl BollingerBands {
    pub fn new(period: usize, std_mult: f64) -> Self {
        assert!(period > 0, "period must be > 0");
        assert!(std_mult >= 0.0, "std_mult must be >= 0");
        Self {
            period,
            std_mult,
            window: RingBuf::new(period, 0.0),
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    #[inline]
    fn update(&mut self, close: f64) -> Option<BollingerOutput> {
        if let Some(old) = self.window.push(close) {
            self.sum -= old;
            self.sum_sq -= old * old;
        }
        self.sum += close;
        self.sum_sq += close * close;

        if self.window.len() < self.period {
            return None;
        }

        let mean = self.sum / self.period as f64;
        let variance = (self.sum_sq / self.period as f64) - mean * mean;
        let std = variance.max(0.0).sqrt();
        Some(BollingerOutput {
            upper: mean + self.std_mult * std,
            middle: mean,
            lower: mean - self.std_mult * std,
        })
    }
}

impl Indicator for BollingerBands {
    type Output = BollingerOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.close)
    }

    fn reset(&mut self) {
        self.window = RingBuf::new(self.period, 0.0);
        self.sum = 0.0;
        self.sum_sq = 0.0;
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
    fn bollinger_emits_after_warmup() {
        let mut bb = BollingerBands::new(3, 2.0);
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

        let outputs: Vec<_> = candles.iter().map(|c| bb.next(c)).collect();
        assert!(outputs
            .iter()
            .take(bb.warmup_period() - 1)
            .all(|o| o.is_none()));
        assert!(outputs
            .iter()
            .skip(bb.warmup_period() - 1)
            .any(|o| o.is_some()));
    }
}
