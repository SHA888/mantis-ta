use super::ATR;
use crate::indicators::{EMA, Indicator};
use crate::types::{Candle, KeltnerOutput};

/// Keltner Channels — EMA-centered volatility bands using ATR for width.
///
/// The middle band is an EMA of closing price; the upper/lower bands
/// offset the middle band by `multiplier * ATR`. Unlike Bollinger Bands
/// (stddev of price), Keltner Channels use true-range-based volatility,
/// so bands react less to single-bar price gaps.
///
/// # Examples
/// ```rust
/// use mantis_ta::indicators::{Indicator, KeltnerChannels};
/// use mantis_ta::types::Candle;
///
/// let candles: Vec<Candle> = (0..10)
///     .map(|i| {
///         let price = 100.0 + i as f64;
///         Candle {
///             timestamp: i as i64,
///             open: price,
///             high: price + 1.0,
///             low: price - 1.0,
///             close: price,
///             volume: 1_000.0,
///         }
///     })
///     .collect();
///
/// // EMA period 5, ATR period 3, multiplier 2.0 -> warmup at max(5, 3) - 1 = 4
/// let out = KeltnerChannels::new(5, 3, 2.0).calculate(&candles);
/// assert!(out.iter().take(4).all(|v| v.is_none()));
/// let k = out[4].unwrap();
/// assert!(k.upper > k.middle && k.middle > k.lower);
/// ```
#[derive(Debug, Clone)]
pub struct KeltnerChannels {
    ema: EMA,
    atr: ATR,
    ema_period: usize,
    atr_period: usize,
    multiplier: f64,
}

impl KeltnerChannels {
    pub fn new(ema_period: usize, atr_period: usize, multiplier: f64) -> Self {
        assert!(ema_period > 0 && atr_period > 0, "periods must be > 0");
        assert!(multiplier > 0.0, "multiplier must be > 0");
        Self {
            ema: EMA::new(ema_period),
            atr: ATR::new(atr_period),
            ema_period,
            atr_period,
            multiplier,
        }
    }
}

impl Indicator for KeltnerChannels {
    type Output = KeltnerOutput;

    fn next(&mut self, candle: &Candle) -> Option<Self::Output> {
        let middle = self.ema.next(candle);
        let atr = self.atr.next(candle);
        let (middle, atr) = match (middle, atr) {
            (Some(m), Some(a)) => (m, a),
            _ => return None,
        };
        let offset = self.multiplier * atr;
        Some(KeltnerOutput {
            upper: middle + offset,
            middle,
            lower: middle - offset,
        })
    }

    fn reset(&mut self) {
        self.ema.reset();
        self.atr.reset();
    }

    fn warmup_period(&self) -> usize {
        self.ema_period.max(self.atr_period)
    }

    fn clone_boxed(&self) -> Box<dyn Indicator<Output = Self::Output>> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(price: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1_000.0,
        }
    }

    #[test]
    fn keltner_emits_after_warmup() {
        let mut kc = KeltnerChannels::new(5, 3, 2.0);
        let candles: Vec<Candle> = (0..8).map(|i| candle(100.0 + i as f64)).collect();

        let outputs: Vec<_> = candles.iter().map(|c| kc.next(c)).collect();
        let wp = kc.warmup_period();
        assert!(outputs.iter().take(wp - 1).all(|o| o.is_none()));
        assert!(outputs[wp - 1].is_some());
    }

    #[test]
    fn keltner_bands_bracket_middle() {
        let mut kc = KeltnerChannels::new(5, 3, 2.0);
        let candles: Vec<Candle> = (0..8).map(|i| candle(100.0 + i as f64)).collect();

        for c in &candles {
            if let Some(out) = kc.next(c) {
                assert!(out.upper > out.middle);
                assert!(out.middle > out.lower);
                assert!((out.upper - out.middle - (out.middle - out.lower)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn keltner_flat_prices_zero_width_bands() {
        // Constant close, but nonzero high/low means ATR still nonzero.
        let mut kc = KeltnerChannels::new(3, 3, 2.0);
        let candles: Vec<Candle> = (0..5)
            .map(|_| Candle {
                timestamp: 0,
                open: 50.0,
                high: 50.0,
                low: 50.0,
                close: 50.0,
                volume: 1_000.0,
            })
            .collect();

        for c in &candles {
            if let Some(out) = kc.next(c) {
                assert!((out.middle - 50.0).abs() < 1e-9);
                assert!((out.upper - 50.0).abs() < 1e-9);
                assert!((out.lower - 50.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn keltner_streaming_matches_batch() {
        let candles: Vec<Candle> = (0..15).map(|i| candle(90.0 + i as f64 * 0.7)).collect();

        let batch = KeltnerChannels::new(5, 4, 1.5).calculate(&candles);

        let mut streamed_kc = KeltnerChannels::new(5, 4, 1.5);
        let streamed: Vec<_> = candles.iter().map(|c| streamed_kc.next(c)).collect();

        assert_eq!(streamed, batch);
    }

    #[test]
    fn keltner_reset_clears_state() {
        let mut kc = KeltnerChannels::new(4, 3, 2.0);
        let candles: Vec<Candle> = (0..6).map(|i| candle(100.0 + i as f64)).collect();
        for c in &candles {
            kc.next(c);
        }
        kc.reset();

        let mut fresh = KeltnerChannels::new(4, 3, 2.0);
        for c in &candles {
            assert_eq!(kc.next(c), fresh.next(c));
        }
    }

    #[test]
    #[should_panic(expected = "periods must be > 0")]
    fn keltner_rejects_zero_period() {
        KeltnerChannels::new(0, 3, 2.0);
    }

    #[test]
    #[should_panic(expected = "multiplier must be > 0")]
    fn keltner_rejects_nonpositive_multiplier() {
        KeltnerChannels::new(5, 3, 0.0);
    }
}
