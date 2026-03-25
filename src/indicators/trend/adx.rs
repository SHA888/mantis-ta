use crate::indicators::Indicator;
use crate::types::{AdxOutput, Candle};
use crate::utils::ringbuf::RingBuf;

/// Average Directional Index measuring trend strength.
///
/// ADX combines +DI and -DI to measure trend strength (0-100).
/// Output includes +DI, -DI, and ADX values.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, ADX};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = (0..50)
///     .map(|i| {
///         let price = 100.0 + i as f64;
///         Candle {
///             timestamp: i as i64,
///             open: price,
///             high: price + 1.0,
///             low: price - 1.0,
///             close: price,
///             volume: 0.0,
///         }
///     })
///     .collect();
///
/// let out = ADX::new(14).calculate(&candles);
/// // Warmup period = 14 * 2 = 28
/// assert!(out.iter().take(27).all(|v| v.is_none()));
/// assert!(out.iter().skip(27).any(|v| v.is_some()));
/// ```
#[derive(Debug, Clone)]
pub struct ADX {
    period: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    tr_sum: f64,
    plus_dm_sum: f64,
    minus_dm_sum: f64,
    bar_count: usize,
    plus_di: Option<f64>,
    minus_di: Option<f64>,
    di_history: RingBuf<f64>,
    adx: Option<f64>,
}

impl ADX {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            tr_sum: 0.0,
            plus_dm_sum: 0.0,
            minus_dm_sum: 0.0,
            bar_count: 0,
            plus_di: None,
            minus_di: None,
            di_history: RingBuf::new(period, 0.0),
            adx: None,
        }
    }

    #[inline]
    fn true_range(high: f64, low: f64, prev_close: f64) -> f64 {
        let hl = high - low;
        let hc = (high - prev_close).abs();
        let lc = (low - prev_close).abs();
        hl.max(hc).max(lc)
    }

    #[inline]
    fn update(&mut self, high: f64, low: f64, close: f64) -> Option<AdxOutput> {
        self.bar_count += 1;

        if let (Some(ph), Some(pl), Some(pc)) = (self.prev_high, self.prev_low, self.prev_close) {
            let tr = Self::true_range(high, low, pc);
            let up_move = high - ph;
            let down_move = pl - low;

            let plus_dm = if up_move > down_move && up_move > 0.0 {
                up_move
            } else {
                0.0
            };

            let minus_dm = if down_move > up_move && down_move > 0.0 {
                down_move
            } else {
                0.0
            };

            if self.bar_count <= self.period {
                self.tr_sum += tr;
                self.plus_dm_sum += plus_dm;
                self.minus_dm_sum += minus_dm;
            } else if self.bar_count == self.period + 1 {
                self.tr_sum = self.tr_sum - self.tr_sum / self.period as f64 + tr;
                self.plus_dm_sum =
                    self.plus_dm_sum - self.plus_dm_sum / self.period as f64 + plus_dm;
                self.minus_dm_sum =
                    self.minus_dm_sum - self.minus_dm_sum / self.period as f64 + minus_dm;
                self.plus_di = Some((self.plus_dm_sum / self.tr_sum) * 100.0);
                self.minus_di = Some((self.minus_dm_sum / self.tr_sum) * 100.0);
            } else {
                self.tr_sum = self.tr_sum - self.tr_sum / self.period as f64 + tr;
                self.plus_dm_sum =
                    self.plus_dm_sum - self.plus_dm_sum / self.period as f64 + plus_dm;
                self.minus_dm_sum =
                    self.minus_dm_sum - self.minus_dm_sum / self.period as f64 + minus_dm;

                let new_plus_di = (self.plus_dm_sum / self.tr_sum) * 100.0;
                let new_minus_di = (self.minus_dm_sum / self.tr_sum) * 100.0;
                self.plus_di = Some(new_plus_di);
                self.minus_di = Some(new_minus_di);

                let di_diff = (new_plus_di - new_minus_di).abs();
                let di_sum = new_plus_di + new_minus_di;
                let dx = if di_sum > 0.0 {
                    (di_diff / di_sum) * 100.0
                } else {
                    0.0
                };

                self.di_history.push(dx);

                match self.bar_count.cmp(&(self.period * 2)) {
                    std::cmp::Ordering::Equal => {
                        let adx_sum: f64 = self.di_history.iter().sum();
                        self.adx = Some(adx_sum / self.period as f64);
                    }
                    std::cmp::Ordering::Greater => {
                        let prev_adx = self.adx.unwrap_or(0.0);
                        self.adx =
                            Some((prev_adx * (self.period - 1) as f64 + dx) / self.period as f64);
                    }
                    std::cmp::Ordering::Less => {}
                }
            }
        }

        self.prev_high = Some(high);
        self.prev_low = Some(low);
        self.prev_close = Some(close);

        if self.bar_count > self.period * 2 {
            Some(AdxOutput {
                plus_di: self.plus_di.unwrap_or(0.0),
                minus_di: self.minus_di.unwrap_or(0.0),
                adx: self.adx.unwrap_or(0.0),
            })
        } else {
            None
        }
    }
}

impl Indicator for ADX {
    type Output = AdxOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.high, candle.low, candle.close)
    }

    fn reset(&mut self) {
        self.prev_high = None;
        self.prev_low = None;
        self.prev_close = None;
        self.tr_sum = 0.0;
        self.plus_dm_sum = 0.0;
        self.minus_dm_sum = 0.0;
        self.bar_count = 0;
        self.plus_di = None;
        self.minus_di = None;
        self.di_history = RingBuf::new(self.period, 0.0);
        self.adx = None;
    }

    fn warmup_period(&self) -> usize {
        self.period * 2
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adx_emits_after_warmup() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| {
                let price = 100.0 + i as f64;
                Candle {
                    timestamp: i as i64,
                    open: price,
                    high: price + 1.0,
                    low: price - 1.0,
                    close: price,
                    volume: 0.0,
                }
            })
            .collect();

        let out = ADX::new(3).calculate(&candles);
        assert!(out.iter().take(5).all(|v| v.is_none()));
        assert!(out.iter().skip(5).any(|v| v.is_some()));
    }
}
