use super::EMA;
use crate::indicators::Indicator;
use crate::types::Candle;

/// Triple Exponential Moving Average over closing prices.
///
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA)), further reducing lag.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, TEMA};
/// use mantis_ta::types::Candle;
///
/// let prices = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
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
/// let out = TEMA::new(3).calculate(&candles);
/// assert!(out.iter().take(6).all(|v| v.is_none()));
/// assert!(out[6].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct TEMA {
    period: usize,
    ema1: EMA,
    ema2: EMA,
    ema3: EMA,
}

impl TEMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            ema1: EMA::new(period),
            ema2: EMA::new(period),
            ema3: EMA::new(period),
        }
    }

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        let ema1_val = self.ema1.next(&Candle {
            timestamp: 0,
            open: value,
            high: value,
            low: value,
            close: value,
            volume: 0.0,
        });

        if let Some(ema1) = ema1_val {
            let ema2_val = self.ema2.next(&Candle {
                timestamp: 0,
                open: ema1,
                high: ema1,
                low: ema1,
                close: ema1,
                volume: 0.0,
            });

            if let Some(ema2) = ema2_val {
                let ema3_val = self.ema3.next(&Candle {
                    timestamp: 0,
                    open: ema2,
                    high: ema2,
                    low: ema2,
                    close: ema2,
                    volume: 0.0,
                });

                if let Some(ema3) = ema3_val {
                    return Some(3.0 * ema1 - 3.0 * ema2 + ema3);
                }
            }
        }

        None
    }
}

impl Indicator for TEMA {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.close)
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
    }

    fn warmup_period(&self) -> usize {
        self.period * 3 - 2
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tema_emits_after_warmup() {
        let prices = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let candles: Vec<Candle> = prices
            .iter()
            .enumerate()
            .map(|(i, p)| Candle {
                timestamp: i as i64,
                open: *p,
                high: *p,
                low: *p,
                close: *p,
                volume: 0.0,
            })
            .collect();

        let out = TEMA::new(3).calculate(&candles);
        assert!(out.iter().take(6).all(|v| v.is_none()));
        assert!(out.iter().skip(6).any(|v| v.is_some()));
    }
}
