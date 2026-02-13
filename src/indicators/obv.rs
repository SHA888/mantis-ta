use crate::indicators::Indicator;
use crate::types::Candle;

/// On-Balance Volume.
#[derive(Debug, Clone)]
pub struct OBV {
    current: f64,
    prev_close: Option<f64>,
}

impl OBV {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            prev_close: None,
        }
    }
}

impl Default for OBV {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for OBV {
    type Output = f64;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        if let Some(prev) = self.prev_close {
            match candle.close.partial_cmp(&prev) {
                Some(std::cmp::Ordering::Greater) => self.current += candle.volume,
                Some(std::cmp::Ordering::Less) => self.current -= candle.volume,
                _ => {}
            }
        }
        self.prev_close = Some(candle.close);
        Some(self.current)
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.prev_close = None;
    }

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
    fn obv_moves_with_direction() {
        let mut obv = OBV::new();
        // close values: 10, 12, 9, 9
        let candles: Vec<Candle> = vec![
            (10.0, 100.0), // first bar, no prev -> OBV = 0
            (12.0, 150.0), // close > prev close -> +150
            (9.0, 80.0),   // close < prev close -> -80
            (9.0, 50.0),   // close == prev close -> unchanged
        ]
        .into_iter()
        .map(|(c, v)| Candle {
            timestamp: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: c,
            volume: v,
        })
        .collect();

        let outputs: Vec<_> = candles.iter().map(|c| obv.next(c)).collect();
        assert_eq!(outputs[0], Some(0.0));   // no previous close
        assert_eq!(outputs[1], Some(150.0)); // 0 + 150
        assert_eq!(outputs[2], Some(70.0));  // 150 - 80
        assert_eq!(outputs[3], Some(70.0));  // unchanged
    }
}
