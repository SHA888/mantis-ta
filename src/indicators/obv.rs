use crate::indicators::Indicator;
use crate::types::Candle;

/// On-Balance Volume.
#[derive(Debug, Clone)]
pub struct OBV {
    current: f64,
}

impl OBV {
    pub fn new() -> Self {
        Self { current: 0.0 }
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
        let vol = candle.volume;
        // OBV starts at 0, then adds/subtracts volume based on close direction.
        if vol == 0.0 {
            return Some(self.current);
        }

        match candle.close.partial_cmp(&candle.open) {
            Some(std::cmp::Ordering::Greater) => self.current += vol,
            Some(std::cmp::Ordering::Less) => self.current -= vol,
            _ => {}
        }
        Some(self.current)
    }

    fn reset(&mut self) {
        self.current = 0.0;
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
        let candles: Vec<Candle> = vec![
            (1.0, 0.0, 10.0), // up -> +10
            (1.0, 2.0, 5.0),  // down -> -5
            (2.0, 2.0, 7.0),  // flat -> 0
        ]
        .into_iter()
        .map(|(o, c, v)| Candle {
            timestamp: 0,
            open: o,
            high: o,
            low: o,
            close: c,
            volume: v,
        })
        .collect();

        let outputs: Vec<_> = candles.iter().map(|c| obv.next(c)).collect();
        assert_eq!(outputs[0], Some(10.0));
        assert_eq!(outputs[1], Some(5.0));
        assert_eq!(outputs[2], Some(5.0));
    }
}
