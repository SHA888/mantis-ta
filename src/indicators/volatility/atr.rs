use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Average True Range using Wilder's smoothing.
#[derive(Debug, Clone)]
pub struct ATR {
    period: usize,
    prev_close: Option<f64>,
    tr_window: RingBuf<f64>,
    sum_tr: f64,
    atr: Option<f64>,
}

impl ATR {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            prev_close: None,
            tr_window: RingBuf::new(period, 0.0),
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
        if let Some(old) = self.tr_window.push(tr) {
            self.sum_tr -= old;
        }
        self.sum_tr += tr;

        let output = if self.atr.is_none() {
            if self.tr_window.len() < self.period {
                None
            } else {
                let initial = self.sum_tr / self.period as f64;
                self.atr = Some(initial);
                self.atr
            }
        } else {
            let prev_atr = self.atr.unwrap();
            let next_atr = (prev_atr * (self.period as f64 - 1.0) + tr) / self.period as f64;
            self.atr = Some(next_atr);
            self.atr
        };

        self.prev_close = Some(candle.close);
        output
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.tr_window = RingBuf::new(self.period, 0.0);
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
        assert!(outputs
            .iter()
            .take(atr.warmup_period())
            .all(|o| o.is_none()));
        assert!(outputs
            .iter()
            .skip(atr.warmup_period())
            .any(|o| o.is_some()));
    }
}
