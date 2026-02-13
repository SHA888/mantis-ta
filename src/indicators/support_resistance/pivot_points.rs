use crate::indicators::Indicator;
use crate::types::{Candle, PivotOutput};

/// Classic floor-trader pivot points calculated from a single candle's OHLC.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, PivotPoints};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = vec![
///     (1.0, 0.5, 0.8),
///     (2.0, 0.5, 1.5),
///     (3.0, 1.0, 2.5),
/// ]
/// .into_iter()
/// .enumerate()
/// .map(|(i, (h, l, c))| Candle {
///     timestamp: i as i64,
///     open: c,
///     high: h,
///     low: l,
///     close: c,
///     volume: 0.0,
/// })
/// .collect();
///
/// let out = PivotPoints::new().calculate(&candles);
/// assert!(out.iter().all(|v| v.is_some()));
/// ```
#[derive(Debug, Clone)]
pub struct PivotPoints;

impl PivotPoints {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PivotPoints {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for PivotPoints {
    type Output = PivotOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        let pp = (candle.high + candle.low + candle.close) / 3.0;
        let r1 = 2.0 * pp - candle.low;
        let s1 = 2.0 * pp - candle.high;
        let r2 = pp + (candle.high - candle.low);
        let s2 = pp - (candle.high - candle.low);
        let r3 = candle.high + 2.0 * (pp - candle.low);
        let s3 = candle.low - 2.0 * (candle.high - pp);

        Some(PivotOutput {
            pp,
            r1,
            r2,
            r3,
            s1,
            s2,
            s3,
        })
    }

    fn reset(&mut self) {}

    fn warmup_period(&self) -> usize {
        0
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_pivots() {
        let mut pivots = PivotPoints::new();
        let candle = Candle {
            timestamp: 0,
            open: 0.0,
            high: 12.0,
            low: 8.0,
            close: 10.0,
            volume: 0.0,
        };
        let out = pivots.next(&candle).unwrap();
        assert_eq!(out.pp, 10.0);
        assert_eq!(out.r1, 12.0);
        assert_eq!(out.s1, 8.0);
    }
}
