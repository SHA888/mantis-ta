//! Golden cross momentum strategy example.
//!
//! Demonstrates building a strategy using the v0.2.0 strategy composition API:
//! - Entry: SMA(20) crosses above SMA(50)
//! - Exit: SMA(20) crosses below SMA(50)
//! - Stop-loss: 2% fixed
//! - Take-profit: 5% fixed
//!
//! The strategy is built using the fluent builder API and serialized to JSON.

use mantis_ta::prelude::*;

fn main() {
    // Build entry condition: SMA(20) crosses above SMA(50)
    let entry = IndicatorRef::sma(20).crosses_above_indicator(IndicatorRef::sma(50));

    // Build exit condition: SMA(20) crosses below SMA(50)
    let exit = IndicatorRef::sma(20).crosses_below_indicator(IndicatorRef::sma(50));

    // Build the strategy using the fluent builder API
    let strategy = Strategy::builder("Golden Cross Momentum")
        .timeframe(Timeframe::D1)
        .entry(entry)
        .exit(exit)
        .stop_loss(StopLoss::FixedPercent(2.0))
        .take_profit(TakeProfit::FixedPercent(5.0))
        .max_position_size_pct(5.0)
        .max_daily_loss_pct(2.0)
        .max_drawdown_pct(10.0)
        .max_concurrent_positions(1)
        .build()
        .expect("Failed to build strategy");

    // Print strategy details
    println!("Strategy: {}", strategy.name);
    println!("Timeframe: {:?}", strategy.timeframe);
    println!("Max position size: {}%", strategy.max_position_size_pct);
    println!("Max daily loss: {}%", strategy.max_daily_loss_pct);
    println!("Max drawdown: {}%", strategy.max_drawdown_pct);
    println!(
        "Max concurrent positions: {}",
        strategy.max_concurrent_positions
    );

    // Serialize to JSON (requires serde feature)
    #[cfg(feature = "serde")]
    {
        match serde_json::to_string_pretty(&strategy) {
            Ok(json) => {
                println!("\nStrategy as JSON:\n{}", json);
            }
            Err(e) => {
                eprintln!("Failed to serialize strategy: {}", e);
            }
        }
    }

    #[cfg(not(feature = "serde"))]
    {
        println!("\nNote: Enable 'serde' feature to serialize strategy to JSON");
    }
}
