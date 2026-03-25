use crate::indicators::Indicator;
use crate::types::Candle;
use crate::utils::ringbuf::RingBuf;

/// Volume Simple Moving Average.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, VolumeSMA};
/// use mantis_ta::types::Candle;
///
/// let vols = [10.0, 20.0, 30.0, 40.0];
/// let candles: Vec<Candle> = vols
///     .iter()
///     .enumerate()
///     .map(|(i, v)| Candle {
///         timestamp: i as i64,
///         open: 0.0,
///         high: 0.0,
///         low: 0.0,
///         close: 0.0,
///         volume: *v,
///     })
///     .collect();
///
/// let out = VolumeSMA::new(3).calculate(&candles);
/// assert!(out.iter().take(2).all(|v| v.is_none()));
/// assert_eq!(out[2], Some((10.0 + 20.0 + 30.0) / 3.0));
/// ```
#[derive(Debug, Clone)]
pub struct VolumeSMA {
    period: usize,
    sum: f64,
    window: RingBuf<f64>,
}

impl VolumeSMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            period,
            sum: 0.0,
            window: RingBuf::new(period, 0.0),
        }
    }

    fn update(&mut self, vol: f64) -> Option<f64> {
        if let Some(old) = self.window.push(vol) {
            self.sum -= old;
        }
        self.sum += vol;
        if self.window.len() < self.period {
            None
        } else {
            Some(self.sum / self.period as f64)
        }
    }
}

impl Indicator for VolumeSMA {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        self.update(candle.volume)
    }

    fn reset(&mut self) {
        self.sum = 0.0;
        self.window = RingBuf::new(self.period, 0.0);
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
    fn volume_sma_emits_after_warmup() {
        let mut v_sma = VolumeSMA::new(3);
        let vols = [10.0, 20.0, 30.0, 40.0];
        let candles: Vec<Candle> = vols
            .iter()
            .map(|v| Candle {
                timestamp: 0,
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: 0.0,
                volume: *v,
            })
            .collect();

        let outputs: Vec<_> = candles.iter().map(|c| v_sma.next(c)).collect();
        assert!(
            outputs
                .iter()
                .take(v_sma.warmup_period() - 1)
                .all(|o| o.is_none())
        );
        assert!(
            outputs
                .iter()
                .skip(v_sma.warmup_period() - 1)
                .any(|o| o.is_some())
        );
    }
}
