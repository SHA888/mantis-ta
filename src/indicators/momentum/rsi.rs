use crate::indicators::Indicator;
use crate::types::Candle;

/// Relative Strength Index (RSI) using Wilder smoothing.
#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
    prev_close: Option<f64>,
    gain_sum: f64,
    loss_sum: f64,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    count: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            prev_close: None,
            gain_sum: 0.0,
            loss_sum: 0.0,
            avg_gain: None,
            avg_loss: None,
            count: 0,
        }
    }

    #[inline]
    fn compute_rsi(&self, avg_gain: f64, avg_loss: f64) -> f64 {
        if avg_loss == 0.0 {
            return 100.0;
        }
        if avg_gain == 0.0 {
            return 0.0;
        }
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
}

impl Indicator for RSI {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        let close = candle.close;

        if let Some(prev) = self.prev_close {
            let change = close - prev;
            let gain = change.max(0.0);
            let loss = (-change).max(0.0);

            if self.avg_gain.is_none() {
                // Warmup accumulation.
                self.gain_sum += gain;
                self.loss_sum += loss;
                self.count += 1;

                if self.count == self.period {
                    let avg_gain = self.gain_sum / self.period as f64;
                    let avg_loss = self.loss_sum / self.period as f64;
                    self.avg_gain = Some(avg_gain);
                    self.avg_loss = Some(avg_loss);
                    self.prev_close = Some(close);
                    return Some(self.compute_rsi(avg_gain, avg_loss));
                }
                self.prev_close = Some(close);
                return None;
            }

            // Wilder smoothing
            let prev_avg_gain = self.avg_gain.unwrap();
            let prev_avg_loss = self.avg_loss.unwrap();
            let new_avg_gain =
                (prev_avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
            let new_avg_loss =
                (prev_avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;

            self.avg_gain = Some(new_avg_gain);
            self.avg_loss = Some(new_avg_loss);
            self.prev_close = Some(close);
            return Some(self.compute_rsi(new_avg_gain, new_avg_loss));
        }

        self.prev_close = Some(close);
        None
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.gain_sum = 0.0;
        self.loss_sum = 0.0;
        self.avg_gain = None;
        self.avg_loss = None;
        self.count = 0;
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
    fn rsi_emits_after_warmup() {
        let mut rsi = RSI::new(3);
        let prices = [1.0, 2.0, 3.0, 2.5, 2.0];
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

        let outputs: Vec<_> = candles.iter().map(|c| rsi.next(c)).collect();
        assert!(outputs
            .iter()
            .take(rsi.warmup_period())
            .all(|o| o.is_none()));
        assert!(outputs
            .iter()
            .skip(rsi.warmup_period())
            .any(|o| o.is_some()));
    }
}
