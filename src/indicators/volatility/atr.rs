use crate::indicators::Indicator;
use crate::types::Candle;

/// Average True Range using Wilder's smoothing.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, ATR};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = [
///     (1.0, 0.5, 0.8),
///     (2.0, 0.5, 1.5),
///     (3.0, 1.0, 2.5),
///     (3.5, 1.5, 3.0),
/// ]
/// .iter()
/// .enumerate()
/// .map(|(i, (h, l, c))| Candle {
///     timestamp: i as i64,
///     open: *c,
///     high: *h,
///     low: *l,
///     close: *c,
///     volume: 0.0,
/// })
/// .collect();
///
/// let out = ATR::new(3).calculate(&candles);
/// assert!(out.iter().take(2).all(|v| v.is_none()));
/// assert!(out[2].is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ATR {
    period: usize,
    prev_close: Option<f64>,
    count: usize,
    sum_tr: f64,
    atr: Option<f64>,
}

impl ATR {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            prev_close: None,
            count: 0,
            sum_tr: 0.0,
            atr: None,
        }
    }

    #[inline]
    fn true_range(&self, candle: &Candle) -> f64 {
        let hl = candle.high - candle.low;
        match self.prev_close {
            None => hl,
            Some(prev) => {
                let h_pc = (candle.high - prev).abs();
                let l_pc = (candle.low - prev).abs();
                hl.max(h_pc).max(l_pc)
            }
        }
    }
}

impl Indicator for ATR {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        let tr = self.true_range(candle);

        let output = if let Some(prev_atr) = self.atr {
            let next_atr = (prev_atr * (self.period as f64 - 1.0) + tr) / self.period as f64;
            self.atr = Some(next_atr);
            self.atr
        } else {
            self.sum_tr += tr;
            self.count += 1;
            if self.count >= self.period {
                let initial = self.sum_tr / self.period as f64;
                self.atr = Some(initial);
                self.atr
            } else {
                None
            }
        };

        self.prev_close = Some(candle.close);
        output
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.count = 0;
        self.sum_tr = 0.0;
        self.atr = None;
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
    fn atr_emits_after_warmup() {
        let mut atr = ATR::new(3);
        let candles: Vec<Candle> = [
            (1.0, 0.5, 0.8),
            (2.0, 0.5, 1.5),
            (3.0, 1.0, 2.5),
            (3.5, 1.5, 3.0),
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

        let outputs: Vec<_> = candles.iter().map(|c| atr.next(c)).collect();
        let wp = atr.warmup_period();
        // First wp-1 outputs should be None
        assert!(outputs.iter().take(wp - 1).all(|o| o.is_none()));
        // Output at index wp-1 should be the first Some
        assert!(outputs[wp - 1].is_some());
    }
}
