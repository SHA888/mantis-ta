use mantis_ta::indicators::{Indicator, MACD, RSI, SMA};
use mantis_ta::types::Candle;

fn sample_candles() -> Vec<Candle> {
    vec![
        Candle {
            timestamp: 0,
            open: 100.0,
            high: 101.0,
            low: 99.5,
            close: 100.5,
            volume: 1_000.0,
        },
        Candle {
            timestamp: 1,
            open: 100.5,
            high: 101.5,
            low: 100.0,
            close: 101.2,
            volume: 1_100.0,
        },
        Candle {
            timestamp: 2,
            open: 101.2,
            high: 102.0,
            low: 100.8,
            close: 101.8,
            volume: 1_050.0,
        },
        Candle {
            timestamp: 3,
            open: 101.8,
            high: 102.2,
            low: 101.0,
            close: 101.1,
            volume: 1_200.0,
        },
        Candle {
            timestamp: 4,
            open: 101.1,
            high: 101.6,
            low: 100.6,
            close: 101.4,
            volume: 1_150.0,
        },
        Candle {
            timestamp: 5,
            open: 101.4,
            high: 101.9,
            low: 100.9,
            close: 101.7,
            volume: 1_180.0,
        },
    ]
}

fn main() {
    let candles = sample_candles();

    // Batch computations (Vec<Option<_>> over the full series)
    let sma_3 = SMA::new(3).calculate(&candles);
    let rsi_3 = RSI::new(3).calculate(&candles);
    let macd = MACD::new(2, 4, 2).calculate(&candles);

    println!("SMA(3): {:?}", sma_3);
    println!("RSI(3): {:?}", rsi_3);
    println!("MACD(2,4,2): {:?}", macd);

    // Streaming usage for a single indicator
    let mut sma = SMA::new(3);
    for (i, c) in candles.iter().enumerate() {
        println!("bar {i}: sma = {:?}", sma.next(c));
    }
}
