//! Backtest a simple momentum strategy with metrics output.
//!
//! This example demonstrates:
//! - Building a momentum strategy (SMA crossover)
//! - Running a backtest with custom configuration
//! - Analyzing performance metrics and trade history
//! - Handling warnings (low trade count, excessive conditions, etc.)
//!
//! Run with: `cargo run --example backtest_momentum --features backtest`

#[cfg(feature = "backtest")]
fn main() {
    use mantis_ta::backtest::{BacktestConfig, ExecutionModel, backtest};
    use mantis_ta::strategy::indicator_ref::IndicatorRef;
    use mantis_ta::strategy::{StopLoss, Strategy};
    use mantis_ta::types::{Candle, Timeframe};
    // Load sample data (5 years of daily data)
    let candles = load_sample_candles();
    println!("Loaded {} candles\n", candles.len());

    // Build a momentum strategy: buy when SMA(50) < 100, sell when SMA(50) > 100
    let strategy = Strategy::builder("momentum_strategy")
        .timeframe(Timeframe::D1)
        .entry(IndicatorRef::sma(50).is_below(100.0))
        .exit(IndicatorRef::sma(50).is_above(100.0))
        .stop_loss(StopLoss::FixedPercent(2.0))
        .build()
        .expect("Failed to build strategy");

    // Configure backtest with realistic costs
    let config = BacktestConfig {
        initial_capital: 100_000.0,
        commission_pct: 0.001, // 0.1% per trade
        slippage_pct: 0.001,   // 0.1% slippage
        execution: ExecutionModel::NextBarOpen,
        ..Default::default()
    };

    // Run backtest
    println!("Running backtest...\n");
    let result = backtest(strategy, &candles, config).expect("Backtest failed");

    // Display results
    println!("=== BACKTEST RESULTS ===\n");
    println!("Starting Capital:  ${:.2}", result.starting_cash);
    println!("Ending Capital:    ${:.2}", result.ending_cash);
    println!(
        "Total Return:      {:.2}%\n",
        result.metrics.total_return * 100.0
    );

    println!("=== PERFORMANCE METRICS ===\n");
    if let Some(cagr) = result.metrics.cagr {
        println!("CAGR:              {:.2}%", cagr * 100.0);
    }
    if let Some(vol) = result.metrics.annualized_vol {
        println!("Annualized Vol:    {:.2}%", vol * 100.0);
    }
    if let Some(sharpe) = result.metrics.sharpe_ratio {
        println!("Sharpe Ratio:      {:.2}", sharpe);
    }
    println!(
        "Max Drawdown:      {:.2}%",
        result.metrics.max_drawdown * 100.0
    );
    println!(
        "Win Rate:          {:.2}%",
        result.metrics.win_rate.unwrap_or(0.0) * 100.0
    );
    if let Some(pf) = result.metrics.profit_factor {
        println!("Profit Factor:     {:.2}", pf);
    }
    println!("Total Trades:      {}", result.metrics.total_trades);
    if let Some(exposure) = result.metrics.exposure_ratio {
        println!("Exposure Ratio:    {:.2}%\n", exposure * 100.0);
    }

    // Display trade history
    println!("=== TRADE HISTORY ===\n");
    for (i, trade) in result.trades.iter().enumerate() {
        let pnl_pct = (trade.pnl / (trade.qty * trade.entry_price)) * 100.0;
        println!(
            "Trade {}: {} @ {:.2} -> {} @ {:.2} | PnL: {:.2} ({:.2}%) | {} bars",
            i + 1,
            trade.qty as i32,
            trade.entry_price,
            trade.qty as i32,
            trade.exit_price,
            trade.pnl,
            pnl_pct,
            trade.holding_period_bars
        );
    }

    // Display warnings
    if !result.warnings.is_empty() {
        println!("\n=== WARNINGS ===\n");
        for warning in &result.warnings {
            println!("Warning: {}", warning);
        }
    }

    // Display sensitivity analysis
    if !result.sensitivity.is_empty() {
        println!("\n=== SENSITIVITY ANALYSIS ===\n");
        println!("Factor | Commission | Slippage | Return");
        println!("-------|------------|----------|--------");
        for sens in &result.sensitivity {
            println!(
                "{:.1}x   | {:.3}%      | {:.3}%    | {:.2}%",
                sens.factor,
                sens.commission_pct * 100.0,
                sens.slippage_pct * 100.0,
                sens.metrics.total_return * 100.0
            );
        }
    }

    // Display walk-forward results
    if let Some(wf) = &result.walk_forward {
        println!("\n=== WALK-FORWARD VALIDATION (70/30 split) ===\n");
        println!(
            "Train Return: {:.2}%",
            wf.train_metrics.total_return * 100.0
        );
        println!("Test Return:  {:.2}%", wf.test_metrics.total_return * 100.0);
        let degradation = (wf.test_metrics.total_return - wf.train_metrics.total_return) * 100.0;
        println!("Degradation:  {:.2}%", degradation);
    }
}

#[cfg(not(feature = "backtest"))]
fn main() {
    println!("This example requires the 'backtest' feature.");
    println!("Run with: cargo run --example backtest_momentum --features backtest");
}

/// Load sample candles for demonstration
#[cfg(feature = "backtest")]
fn load_sample_candles() -> Vec<mantis_ta::types::Candle> {
    use mantis_ta::types::Candle;

    // Generate synthetic daily data for demonstration
    let mut candles = Vec::new();
    let mut price = 100.0;
    let mut timestamp = 1609459200i64; // 2021-01-01

    for i in 0..1260 {
        // ~5 years of daily data
        // Simulate price movement with trend and noise
        let trend = (i as f64 / 1260.0) * 50.0; // Uptrend
        let noise = (i as f64 * 0.1).sin() * 5.0; // Oscillation
        price = 100.0 + trend + noise + (i as f64 * 0.01);

        let open = price * 0.99;
        let close = price;
        let high = price * 1.02;
        let low = price * 0.98;
        let volume = 1_000_000.0;

        candles.push(Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });

        timestamp += 86400; // +1 day in seconds
    }

    candles
}
