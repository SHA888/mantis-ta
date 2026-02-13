use mantis_ta::indicators::{Indicator, EMA};
use mantis_ta::types::Candle;

fn main() {
    let mut ema = EMA::new(5);
    let prices = [100.0, 101.0, 102.0, 101.5, 101.8, 102.2, 103.0];

    for (i, price) in prices.iter().enumerate() {
        let candle = Candle {
            timestamp: i as i64,
            open: *price,
            high: *price,
            low: *price,
            close: *price,
            volume: 0.0,
        };
        let out = ema.next(&candle);
        println!("bar {i}: close={price:.2} ema={:?}", out);
    }
}
