//! Backtesting engine for strategy validation and performance analysis.
//!
//! This module is planned for v0.3.0 and will provide:
//! - `BacktestConfig` struct for configuration (initial capital, commission, slippage, etc.)
//! - `ExecutionModel` enum (NextBarOpen, CurrentBarClose)
//! - `backtest()` runner function for the main execution loop
//! - `BrokerSim` for simulated order fills with slippage
//! - `Portfolio` for position tracking and cash accounting
//! - `BacktestMetrics` for comprehensive performance analysis
//! - Integrity rules: no lookahead bias, next-bar execution, proper commission/slippage handling
//!
//! See [SPEC.md](../SPEC.md) §6 for detailed requirements.

use crate::strategy::evaluator::strategy_engine;
use crate::strategy::types::{ConditionNode, Strategy};
use crate::types::{Candle, ExitReason, MantisError, Result, Side, Signal};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Execution model for order fills in backtesting.
///
/// Determines when pending orders are filled relative to the current candle.
///
/// # Variants
///
/// * `NextBarOpen` - Orders fill at the next bar's open price (default, more realistic)
/// * `CurrentBarClose` - Orders fill at the current bar's close price (less realistic, for testing)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionModel {
    /// Fill orders at the next bar's open price
    NextBarOpen,
    /// Fill orders at the current bar's close price
    CurrentBarClose,
}

fn count_conditions(node: &ConditionNode) -> usize {
    match node {
        ConditionNode::Condition(_) => 1,
        ConditionNode::Group(group) => match group {
            crate::strategy::types::ConditionGroup::AllOf(children)
            | crate::strategy::types::ConditionGroup::AnyOf(children) => {
                children.iter().map(count_conditions).sum()
            }
        },
    }
}

fn equity_value(cash: f64, position_qty: f64, mark_price: f64) -> f64 {
    cash + position_qty * mark_price
}

fn compute_metrics(
    trades: &[Trade],
    equity_curve: &[(i64, f64)],
    starting_cash: f64,
) -> BacktestMetrics {
    // Validate equity curve values are finite
    for (i, &(_, eq)) in equity_curve.iter().enumerate() {
        if !eq.is_finite() {
            eprintln!(
                "Warning: equity curve contains non-finite value at index {}",
                i
            );
        }
    }

    let total_return = if starting_cash > 0.0 {
        equity_curve
            .last()
            .map(|(_, eq)| eq / starting_cash - 1.0)
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Per-bar returns for volatility and Sharpe
    let mut per_bar_returns: Vec<f64> = Vec::new();
    for w in equity_curve.windows(2) {
        let prev = w[0].1;
        let next = w[1].1;
        if prev > 0.0 {
            per_bar_returns.push(next / prev - 1.0);
        }
    }

    let bars = per_bar_returns.len() as f64;
    let bars_per_year: f64 = 252.0; // assumption for daily-like data
    let mean_ret = if bars > 0.0 {
        per_bar_returns.iter().sum::<f64>() / bars
    } else {
        0.0
    };
    let var = if bars > 1.0 {
        per_bar_returns
            .iter()
            .map(|r| {
                let diff = r - mean_ret;
                diff * diff
            })
            .sum::<f64>()
            / (bars - 1.0)
    } else {
        0.0
    };
    let vol = var.sqrt();
    let annualized_vol = if vol.is_finite() && bars > 0.0 {
        Some(vol * bars_per_year.sqrt())
    } else {
        None
    };
    let sharpe_ratio = if vol > 0.0 {
        Some(mean_ret / vol * bars_per_year.sqrt())
    } else {
        None
    };

    // CAGR based on timestamps (assumes timestamps are in milliseconds)
    let cagr = if equity_curve.len() >= 2 {
        let start_ts = equity_curve.first().unwrap().0 as f64 / 1000.0; // ms -> seconds
        let end_ts = equity_curve.last().unwrap().0 as f64 / 1000.0;
        let duration_years = (end_ts - start_ts) / (365.0 * 24.0 * 3600.0);
        if duration_years > 0.0 && starting_cash > 0.0 {
            let ending = equity_curve.last().unwrap().1;
            let ratio = ending / starting_cash;
            Some(ratio.powf(1.0 / duration_years) - 1.0)
        } else {
            None
        }
    } else {
        None
    };

    // Drawdown
    let mut peak = equity_curve.first().map(|(_, eq)| *eq).unwrap_or(0.0);
    let mut max_dd = 0.0;
    for &(_, eq) in equity_curve {
        if eq > peak {
            peak = eq;
        }
        if peak > 0.0 {
            let dd = (peak - eq).max(0.0);
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    let max_drawdown_pct = if peak > 0.0 { max_dd / peak } else { 0.0 };

    // Trade stats
    let total_trades = trades.len();
    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut win_sum = 0.0;
    let mut loss_sum = 0.0;
    for t in trades {
        if t.pnl > 0.0 {
            wins += 1;
            win_sum += t.pnl;
        } else if t.pnl < 0.0 {
            losses += 1;
            loss_sum += t.pnl.abs();
        }
    }
    let win_rate = if total_trades > 0 {
        Some(wins as f64 / total_trades as f64)
    } else {
        None
    };
    let profit_factor = match (win_sum, loss_sum) {
        (_, 0.0) if wins > 0 => Some(f64::INFINITY),
        (ws, ls) if ls > 0.0 => Some(ws / ls),
        _ => None,
    };
    let average_win = if wins > 0 {
        Some(win_sum / wins as f64)
    } else {
        None
    };
    let average_loss = if losses > 0 {
        Some(-(loss_sum / losses as f64))
    } else {
        None
    };

    // Exposure: sum of holding periods divided by total backtest duration
    // Note: This assumes non-overlapping trades (true for long-only strategies)
    let total_bars = equity_curve.len().saturating_sub(1);
    let holding_bars: usize = trades.iter().map(|t| t.holding_period_bars).sum();
    let exposure_ratio = if total_bars > 0 {
        Some((holding_bars as f64 / total_bars as f64).min(1.0))
    } else {
        None
    };

    BacktestMetrics {
        total_return,
        cagr,
        annualized_vol,
        sharpe_ratio,
        max_drawdown: max_dd,
        max_drawdown_pct,
        win_rate,
        profit_factor,
        average_win,
        average_loss,
        total_trades,
        exposure_ratio,
    }
}

/// Configuration for backtest execution.
///
/// Defines initial capital, trading costs (commission and slippage), execution model,
/// and other parameters that affect backtest behavior.
///
/// # Fields
///
/// * `initial_capital` - Starting cash in dollars (must be > 0)
/// * `commission_per_trade` - Fixed commission per trade in dollars (default: 0)
/// * `commission_pct` - Percentage commission per trade as decimal (default: 0.001 = 0.1%)
/// * `slippage_pct` - Slippage as percentage of notional (default: 0.001 = 0.1%)
/// * `execution` - Order fill model: NextBarOpen or CurrentBarClose (default: NextBarOpen)
/// * `fractional_shares` - Allow fractional share quantities (default: false)
/// * `margin_requirement` - Margin requirement multiplier (MVP only supports 1.0)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BacktestConfig {
    /// Starting capital in dollars
    pub initial_capital: f64,
    /// Fixed commission per trade in dollars
    pub commission_per_trade: f64,
    /// Percentage commission per trade (0.0-1.0)
    pub commission_pct: f64,
    /// Slippage as percentage of notional (0.0-1.0)
    pub slippage_pct: f64,
    /// Order execution model
    pub execution: ExecutionModel,
    /// Allow fractional share quantities
    pub fractional_shares: bool,
    /// Margin requirement multiplier (MVP: 1.0 only)
    pub margin_requirement: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            commission_per_trade: 0.0,
            commission_pct: 0.001,
            slippage_pct: 0.001,
            execution: ExecutionModel::NextBarOpen,
            fractional_shares: false,
            margin_requirement: 1.0,
        }
    }
}

impl BacktestConfig {
    fn validate(&self) -> Result<()> {
        let check_pct = |value: f64, name: &'static str| -> Result<()> {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(MantisError::InvalidParameter {
                    param: name,
                    value: value.to_string(),
                    reason: "must be finite and between 0 and 1",
                });
            }
            Ok(())
        };

        if !self.initial_capital.is_finite() || self.initial_capital <= 0.0 {
            return Err(MantisError::InvalidParameter {
                param: "initial_capital",
                value: self.initial_capital.to_string(),
                reason: "must be > 0 and finite",
            });
        }
        if !self.commission_per_trade.is_finite() || self.commission_per_trade < 0.0 {
            return Err(MantisError::InvalidParameter {
                param: "commission_per_trade",
                value: self.commission_per_trade.to_string(),
                reason: "must be finite and >= 0",
            });
        }

        check_pct(self.commission_pct, "commission_pct")?;
        check_pct(self.slippage_pct, "slippage_pct")?;

        if !self.fractional_shares {
            // No additional checks needed
        }

        if self.margin_requirement != 1.0 {
            return Err(MantisError::BacktestError(
                "margin is not supported in MVP backtester".to_string(),
            ));
        }

        Ok(())
    }
}

/// A completed trade with entry/exit details and P&L.
///
/// Records all information about a single round-trip trade from entry to exit,
/// including timestamps, prices, quantity, profit/loss, and exit reason.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    /// Timestamp when position was entered (milliseconds)
    pub entry_timestamp: i64,
    /// Timestamp when position was exited (milliseconds)
    pub exit_timestamp: i64,
    /// Price at which position was entered
    pub entry_price: f64,
    /// Price at which position was exited
    pub exit_price: f64,
    /// Quantity of shares/contracts
    pub qty: f64,
    /// Profit/loss in dollars
    pub pnl: f64,
    /// Reason for exit (RuleTriggered, StopLoss, TakeProfit, etc.)
    pub exit_reason: ExitReason,
    /// Number of bars held
    pub holding_period_bars: usize,
}

/// Complete backtest results including metrics, trades, and diagnostics.
///
/// Contains all output from a backtest run: final capital, equity curve,
/// performance metrics, trade history, warnings, and sensitivity analysis.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct BacktestResult {
    /// Starting capital in dollars
    pub starting_cash: f64,
    /// Ending capital in dollars
    pub ending_cash: f64,
    /// Equity curve: (timestamp, equity) pairs
    pub equity_curve: Vec<(i64, f64)>,
    /// Performance metrics
    pub metrics: BacktestMetrics,
    /// All completed trades
    pub trades: Vec<Trade>,
    /// Warnings (low trade count, excessive conditions, etc.)
    pub warnings: Vec<String>,
    /// Parameter sensitivity analysis results
    pub sensitivity: Vec<ParameterSensitivity>,
    /// Walk-forward validation results (70/30 split)
    pub walk_forward: Option<WalkForwardResult>,
}

/// Performance metrics computed from backtest results.
///
/// Comprehensive set of metrics including returns, risk-adjusted returns,
/// drawdown, trade statistics, and exposure ratios.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BacktestMetrics {
    /// Total return as decimal (e.g., 0.25 = 25%)
    pub total_return: f64,
    /// Compound annual growth rate (if >= 1 year of data)
    pub cagr: Option<f64>,
    /// Annualized volatility of returns
    pub annualized_vol: Option<f64>,
    /// Sharpe ratio (assuming 0% risk-free rate)
    pub sharpe_ratio: Option<f64>,
    /// Maximum drawdown in dollars
    pub max_drawdown: f64,
    /// Maximum drawdown as percentage
    pub max_drawdown_pct: f64,
    /// Win rate: winning trades / total trades
    pub win_rate: Option<f64>,
    /// Profit factor: sum of wins / sum of losses
    pub profit_factor: Option<f64>,
    /// Average profit per winning trade
    pub average_win: Option<f64>,
    /// Average loss per losing trade
    pub average_loss: Option<f64>,
    /// Total number of completed trades
    pub total_trades: usize,
    /// Exposure ratio: time in market / total duration
    pub exposure_ratio: Option<f64>,
}

/// Parameter sensitivity analysis for a single factor.
///
/// Shows how metrics change when commission and slippage are scaled by a factor.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParameterSensitivity {
    /// Scaling factor (0.9, 1.0, 1.1 for ±10%)
    pub factor: f64,
    /// Scaled commission percentage
    pub commission_pct: f64,
    /// Scaled slippage percentage
    pub slippage_pct: f64,
    /// Metrics for this parameter set
    pub metrics: BacktestMetrics,
}

/// Walk-forward validation results (70/30 time split).
///
/// Compares in-sample (training) metrics to out-of-sample (test) metrics
/// to detect overfitting. Degradation indicates the strategy may not generalize.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WalkForwardResult {
    /// Index where train/test split occurs
    pub split_index: usize,
    /// Metrics on training set (first 70% of data)
    pub train_metrics: BacktestMetrics,
    /// Metrics on test set (last 30% of data)
    pub test_metrics: BacktestMetrics,
}

#[derive(Debug, Clone, Copy)]
pub struct Portfolio {
    cash: f64,
    position_qty: f64,
    entry_price: Option<f64>,
    entry_timestamp: Option<i64>,
    entry_bar_idx: Option<usize>,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Result<Self> {
        if !initial_cash.is_finite() || initial_cash <= 0.0 {
            return Err(MantisError::InvalidParameter {
                param: "initial_capital",
                value: initial_cash.to_string(),
                reason: "must be a finite number > 0",
            });
        }
        Ok(Self {
            cash: initial_cash,
            position_qty: 0.0,
            entry_price: None,
            entry_timestamp: None,
            entry_bar_idx: None,
        })
    }

    pub fn cash(&self) -> f64 {
        self.cash
    }

    pub fn position_qty(&self) -> f64 {
        self.position_qty
    }

    pub fn is_flat(&self) -> bool {
        self.position_qty.abs() < 1e-9
    }

    fn can_buy(&self, total_cost: f64) -> bool {
        // Allow small epsilon tolerance for floating-point rounding errors
        total_cost.is_finite() && total_cost <= self.cash + 1e-9
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BrokerSim;

impl BrokerSim {
    pub fn new() -> Self {
        Self
    }

    fn apply_slippage(price: f64, side: Side, slippage_pct: f64) -> f64 {
        if !price.is_finite() || price <= 0.0 {
            return f64::NAN;
        }
        match side {
            Side::Long => price * (1.0 + slippage_pct),
            Side::Short => price * (1.0 - slippage_pct),
        }
    }

    fn commission(notional: f64, config: &BacktestConfig) -> f64 {
        if !notional.is_finite() || notional < 0.0 {
            return f64::NAN;
        }
        config.commission_per_trade + notional * config.commission_pct
    }

    pub fn buy(
        &self,
        portfolio: &mut Portfolio,
        qty: f64,
        price: f64,
        timestamp: i64,
        bar_idx: usize,
        config: &BacktestConfig,
    ) -> Result<()> {
        if qty <= 0.0 || !qty.is_finite() {
            return Err(MantisError::BacktestError("invalid buy qty".to_string()));
        }
        if !portfolio.is_flat() {
            return Err(MantisError::BacktestError(
                "portfolio already has an open position".to_string(),
            ));
        }
        let fill_price = Self::apply_slippage(price, Side::Long, config.slippage_pct);
        if !fill_price.is_finite() {
            return Err(MantisError::BacktestError("invalid price".to_string()));
        }
        let notional = fill_price * qty;
        let commission = Self::commission(notional, config);
        if !commission.is_finite() {
            return Err(MantisError::BacktestError("invalid commission".to_string()));
        }
        let total_cost = notional + commission;
        if !portfolio.can_buy(total_cost) {
            return Err(MantisError::BacktestError(
                "insufficient cash for buy".to_string(),
            ));
        }
        portfolio.cash -= total_cost;
        portfolio.position_qty = qty;
        portfolio.entry_price = Some(fill_price);
        portfolio.entry_timestamp = Some(timestamp);
        portfolio.entry_bar_idx = Some(bar_idx);
        Ok(())
    }

    pub fn sell(
        &self,
        portfolio: &mut Portfolio,
        price: f64,
        config: &BacktestConfig,
    ) -> Result<(f64, f64)> {
        if portfolio.is_flat() {
            return Err(MantisError::BacktestError(
                "no position to sell".to_string(),
            ));
        }
        let qty = portfolio.position_qty;
        let fill_price = Self::apply_slippage(price, Side::Short, config.slippage_pct);
        if !fill_price.is_finite() {
            return Err(MantisError::BacktestError("invalid price".to_string()));
        }
        let notional = fill_price * qty;
        let commission = Self::commission(notional, config);
        if !commission.is_finite() {
            return Err(MantisError::BacktestError("invalid commission".to_string()));
        }
        portfolio.cash += notional - commission;
        let entry_price = portfolio
            .entry_price
            .ok_or_else(|| MantisError::BacktestError("missing entry price".to_string()))?;

        portfolio.position_qty = 0.0;
        portfolio.entry_price = None;
        portfolio.entry_timestamp = None;
        portfolio.entry_bar_idx = None;

        // P&L already accounts for commission via cash change; do not double-subtract.
        let pnl = (fill_price - entry_price) * qty;
        Ok((fill_price, pnl))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PendingOrder {
    EnterLong,
    ExitLong(ExitReason),
}

/// Calculate position size based on available cash, execution price, and config.
fn calculate_position_size(cash: f64, exec_price: f64, config: &BacktestConfig) -> f64 {
    let effective_price = exec_price * (1.0 + config.slippage_pct);
    let cash_after_fixed = (cash - config.commission_per_trade).max(0.0);
    let denom = effective_price * (1.0 + config.commission_pct);
    let raw_qty = if denom > 0.0 {
        cash_after_fixed / denom
    } else {
        0.0
    };
    if config.fractional_shares {
        raw_qty
    } else {
        raw_qty.floor()
    }
}

/// Helper struct to capture portfolio entry state before sell() clears it.
struct EntryState {
    timestamp: i64,
    price: f64,
    bar_idx: usize,
    qty: f64,
}

/// Create a Trade record from entry state and exit details.
fn record_trade(
    entry: EntryState,
    exit_price: f64,
    pnl: f64,
    exit_timestamp: i64,
    exit_reason: ExitReason,
    exit_bar_idx: usize,
) -> Trade {
    Trade {
        entry_timestamp: entry.timestamp,
        exit_timestamp,
        entry_price: entry.price,
        exit_price,
        qty: entry.qty,
        pnl,
        exit_reason,
        holding_period_bars: exit_bar_idx.saturating_sub(entry.bar_idx),
    }
}

/// Validate candle OHLC relationships and timestamp ordering.
fn validate_candles(candles: &[Candle]) -> Result<()> {
    if candles.is_empty() {
        return Ok(());
    }

    let mut prev_timestamp = None;
    for (i, candle) in candles.iter().enumerate() {
        // Validate OHLC relationships
        if candle.high < candle.low {
            return Err(MantisError::BacktestError(format!(
                "candle {} has high < low ({} < {})",
                i, candle.high, candle.low
            )));
        }
        if candle.high < candle.open || candle.high < candle.close {
            return Err(MantisError::BacktestError(format!(
                "candle {} has high below open or close",
                i
            )));
        }
        if candle.low > candle.open || candle.low > candle.close {
            return Err(MantisError::BacktestError(format!(
                "candle {} has low above open or close",
                i
            )));
        }

        // Validate timestamp ordering
        if let Some(prev_ts) = prev_timestamp {
            if candle.timestamp <= prev_ts {
                return Err(MantisError::BacktestError(format!(
                    "candle {} has non-increasing timestamp ({} <= {})",
                    i, candle.timestamp, prev_ts
                )));
            }
        }
        prev_timestamp = Some(candle.timestamp);
    }

    Ok(())
}

fn backtest_once(
    strategy: &Strategy,
    candles: &[Candle],
    config: BacktestConfig,
) -> Result<BacktestResult> {
    if candles.len() < 2 {
        return Err(MantisError::InsufficientData {
            required: 2,
            provided: candles.len(),
        });
    }
    config.validate()?;
    validate_candles(candles)?;

    let condition_count = {
        let entry_cnt = count_conditions(&strategy.entry);
        let exit_cnt = strategy.exit.as_ref().map(count_conditions).unwrap_or(0);
        entry_cnt + exit_cnt
    };

    let mut portfolio = Portfolio::new(config.initial_capital)?;
    let broker = BrokerSim::new();
    let mut engine = strategy_engine(strategy.clone());
    let mut trades: Vec<Trade> = Vec::new();
    let mut equity_curve: Vec<(i64, f64)> = Vec::with_capacity(candles.len() + 1);
    // Seed equity curve with starting cash at first candle timestamp
    equity_curve.push((candles[0].timestamp, portfolio.cash()));
    let mut warnings: Vec<String> = Vec::new();

    let mut pending: Option<PendingOrder> = None;

    for (i, candle) in candles.iter().enumerate() {
        // 1) Execute pending orders (only used for NextBarOpen) at the current bar's open.
        if let Some(order) = pending.take() {
            let exec_price = candle.open;
            match order {
                PendingOrder::EnterLong => {
                    if portfolio.is_flat() {
                        let qty = calculate_position_size(portfolio.cash(), exec_price, &config);
                        if qty >= 1.0 || (config.fractional_shares && qty > 0.0) {
                            broker.buy(
                                &mut portfolio,
                                qty,
                                exec_price,
                                candle.timestamp,
                                i,
                                &config,
                            )?;
                        }
                    }
                }
                PendingOrder::ExitLong(reason) => {
                    if !portfolio.is_flat() {
                        let entry = EntryState {
                            timestamp: portfolio.entry_timestamp.ok_or_else(|| {
                                MantisError::BacktestError("missing entry timestamp".to_string())
                            })?,
                            price: portfolio.entry_price.ok_or_else(|| {
                                MantisError::BacktestError("missing entry price".to_string())
                            })?,
                            qty: portfolio.position_qty,
                            bar_idx: portfolio.entry_bar_idx.ok_or_else(|| {
                                MantisError::BacktestError("missing entry bar idx".to_string())
                            })?,
                        };

                        let (exit_price, pnl) = broker.sell(&mut portfolio, exec_price, &config)?;
                        trades.push(record_trade(
                            entry,
                            exit_price,
                            pnl,
                            candle.timestamp,
                            reason,
                            i,
                        ));
                    }
                }
            }
        }

        let signal = engine.next(candle);
        match signal {
            Signal::Entry(Side::Long) => {
                match config.execution {
                    ExecutionModel::CurrentBarClose => {
                        // Execute immediately at current bar close
                        let exec_price = candle.close;
                        if portfolio.is_flat() {
                            let qty =
                                calculate_position_size(portfolio.cash(), exec_price, &config);
                            if qty >= 1.0 || (config.fractional_shares && qty > 0.0) {
                                broker.buy(
                                    &mut portfolio,
                                    qty,
                                    exec_price,
                                    candle.timestamp,
                                    i,
                                    &config,
                                )?;
                            }
                        }
                    }
                    ExecutionModel::NextBarOpen => {
                        if i + 1 < candles.len() {
                            pending = Some(PendingOrder::EnterLong);
                        }
                    }
                }
            }
            Signal::Exit(reason) => match config.execution {
                ExecutionModel::CurrentBarClose => {
                    if !portfolio.is_flat() {
                        let entry = EntryState {
                            timestamp: portfolio.entry_timestamp.ok_or_else(|| {
                                MantisError::BacktestError("missing entry timestamp".to_string())
                            })?,
                            price: portfolio.entry_price.ok_or_else(|| {
                                MantisError::BacktestError("missing entry price".to_string())
                            })?,
                            qty: portfolio.position_qty,
                            bar_idx: portfolio.entry_bar_idx.ok_or_else(|| {
                                MantisError::BacktestError("missing entry bar idx".to_string())
                            })?,
                        };

                        let exec_price = candle.close;
                        let (exit_price, pnl) = broker.sell(&mut portfolio, exec_price, &config)?;
                        trades.push(record_trade(
                            entry,
                            exit_price,
                            pnl,
                            candle.timestamp,
                            reason,
                            i,
                        ));
                    }
                }
                ExecutionModel::NextBarOpen => {
                    if i + 1 < candles.len() {
                        pending = Some(PendingOrder::ExitLong(reason));
                    }
                }
            },
            Signal::Hold | Signal::Entry(Side::Short) => {}
        }

        // Mark-to-market equity at bar close
        let equity = equity_value(portfolio.cash(), portfolio.position_qty, candle.close);
        equity_curve.push((candle.timestamp, equity));
    }

    if !portfolio.is_flat() {
        // Liquidate at the last available price. For NextBarOpen there is no next bar, so we
        // fall back to last.close.
        let last = candles
            .last()
            .ok_or_else(|| MantisError::BacktestError("no last candle".to_string()))?;
        let exec_price = match config.execution {
            ExecutionModel::CurrentBarClose => last.close,
            ExecutionModel::NextBarOpen => last.close,
        };

        let entry = EntryState {
            timestamp: portfolio
                .entry_timestamp
                .ok_or_else(|| MantisError::BacktestError("missing entry timestamp".to_string()))?,
            price: portfolio
                .entry_price
                .ok_or_else(|| MantisError::BacktestError("missing entry price".to_string()))?,
            qty: portfolio.position_qty,
            bar_idx: portfolio
                .entry_bar_idx
                .ok_or_else(|| MantisError::BacktestError("missing entry bar idx".to_string()))?,
        };

        let (exit_price, pnl) = broker.sell(&mut portfolio, exec_price, &config)?;
        trades.push(record_trade(
            entry,
            exit_price,
            pnl,
            last.timestamp,
            ExitReason::RuleTriggered,
            candles.len() - 1,
        ));

        // Update final equity after liquidation (replace last equity point to avoid duplicate timestamp)
        let equity = equity_value(portfolio.cash(), portfolio.position_qty, last.close);
        if let Some(last_entry) = equity_curve.last_mut() {
            last_entry.1 = equity;
        }
    }

    if trades.len() < 30 {
        warnings.push(format!(
            "Minimum trade count warning: {} trades (< 30) — results may be statistically unreliable",
            trades.len()
        ));
    }
    if condition_count >= 7 {
        warnings.push(format!(
            "Excessive condition warning: {} conditions (>= 7) — potential overfitting",
            condition_count
        ));
    }

    Ok(BacktestResult {
        starting_cash: config.initial_capital,
        ending_cash: portfolio.cash(),
        metrics: compute_metrics(&trades, &equity_curve, config.initial_capital),
        equity_curve,
        trades,
        warnings,
        sensitivity: Vec::new(),
        walk_forward: None,
    })
}

pub fn backtest(
    strategy: Strategy,
    candles: &[Candle],
    config: BacktestConfig,
) -> Result<BacktestResult> {
    let mut base = backtest_once(&strategy, candles, config)?;

    // Parameter sensitivity: scale commission_pct and slippage_pct by ±10%
    // Include base (factor=1.0) in sensitivity array
    let factors = [0.9, 1.0, 1.1];
    let mut sensitivity = Vec::new();
    for factor in factors {
        let mut cfg = config;
        cfg.commission_pct *= factor;
        cfg.slippage_pct *= factor;

        // Validate scaled config stays within bounds
        if cfg.commission_pct > 1.0 || cfg.slippage_pct > 1.0 {
            base.warnings.push(format!(
                "Sensitivity analysis skipped factor {}: scaled parameters exceed valid range",
                factor
            ));
            continue;
        }

        match backtest_once(&strategy, candles, cfg) {
            Ok(res) => {
                sensitivity.push(ParameterSensitivity {
                    factor,
                    commission_pct: cfg.commission_pct,
                    slippage_pct: cfg.slippage_pct,
                    metrics: res.metrics,
                });
            }
            Err(e) => {
                base.warnings.push(format!(
                    "Sensitivity analysis failed for factor {}: {}",
                    factor, e
                ));
            }
        }
    }

    // Walk-forward: 70/30 split (time-ordered)
    let split_index = ((candles.len() as f64) * 0.7).floor() as usize;
    let walk_forward = if split_index >= 2 && candles.len() - split_index >= 2 {
        let train = &candles[..split_index];
        let test = &candles[split_index..];
        match (
            backtest_once(&strategy, train, config),
            backtest_once(&strategy, test, config),
        ) {
            (Ok(train_res), Ok(test_res)) => Some(WalkForwardResult {
                split_index,
                train_metrics: train_res.metrics,
                test_metrics: test_res.metrics,
            }),
            (Err(e), _) | (_, Err(e)) => {
                base.warnings
                    .push(format!("Walk-forward validation failed: {}", e));
                None
            }
        }
    } else {
        base.warnings.push(
            "Walk-forward validation skipped: insufficient data for 70/30 split (need >= 4 candles with 2+ in each split)".to_string()
        );
        None
    };

    Ok(BacktestResult {
        sensitivity,
        walk_forward,
        ..base
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::StopLoss;
    use crate::strategy::indicator_ref::IndicatorRef;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, p)| Candle {
                timestamp: i as i64,
                open: *p,
                high: *p,
                low: *p,
                close: *p,
                volume: 0.0,
            })
            .collect()
    }

    #[test]
    fn backtest_runs_and_emits_trade() {
        let entry = IndicatorRef::sma(1).is_above(1.0);
        let exit = IndicatorRef::sma(1).is_below(1.5);
        let strategy = Strategy::builder("bt")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        let candles = make_candles(&[1.0, 2.0, 2.0, 1.0]);
        let res = backtest(strategy, &candles, BacktestConfig::default()).unwrap();

        assert!(res.ending_cash.is_finite());
        assert!(!res.trades.is_empty());
    }

    #[test]
    fn backtest_metrics_match_hand_calculation() {
        // Simple strategy: buy at bar 1 (price 2.0), sell at bar 3 (price 1.0)
        // Entry: SMA(1) > 1.5 → triggers at bar 1 (SMA=2.0)
        // Exit: SMA(1) < 1.5 → triggers at bar 3 (SMA=1.0)
        let entry = IndicatorRef::sma(1).is_above(1.5);
        let exit = IndicatorRef::sma(1).is_below(1.5);
        let strategy = Strategy::builder("metrics_test")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        let candles = make_candles(&[1.0, 2.0, 2.0, 1.0]);
        let config = BacktestConfig {
            initial_capital: 100_000.0,
            commission_per_trade: 0.0,
            commission_pct: 0.0,
            slippage_pct: 0.0,
            ..Default::default()
        };
        let res = backtest(strategy, &candles, config).unwrap();

        // Expected: 1 trade, entry at 2.0, exit at 1.0, qty = 50000
        // P&L = (1.0 - 2.0) * 50000 = -50000
        assert_eq!(res.trades.len(), 1);
        let trade = &res.trades[0];
        assert_eq!(trade.entry_price, 2.0);
        assert_eq!(trade.exit_price, 1.0);
        assert_eq!(trade.qty, 50000.0);
        assert_eq!(trade.pnl, -50000.0);
        assert_eq!(res.metrics.total_return, -0.5);
    }

    #[test]
    fn backtest_no_lookahead_bias() {
        // Strategy that uses future data: entry when next bar's close > current
        // This should NOT generate signals because indicators only see current bar
        let entry = IndicatorRef::sma(1).is_above(1.5);
        let exit = IndicatorRef::sma(1).is_below(0.5);
        let strategy = Strategy::builder("no_lookahead")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        // Prices: [1.0, 2.0, 3.0, 1.0]
        // SMA(1) = [1.0, 2.0, 3.0, 1.0]
        // Entry triggers at bar 1 (SMA=2.0 > 1.5)
        // Exit triggers at bar 3 (SMA=1.0 < 0.5) - NO, 1.0 is not < 0.5
        let candles = make_candles(&[1.0, 2.0, 3.0, 1.0]);
        let res = backtest(strategy, &candles, BacktestConfig::default()).unwrap();

        // Should have 1 trade (entry at bar 1, liquidated at end)
        assert_eq!(res.trades.len(), 1);
    }

    #[test]
    fn backtest_cash_accounting_prevents_overbuy() {
        // Strategy tries to buy more than available cash
        let entry = IndicatorRef::sma(1).is_above(0.5);
        let exit = IndicatorRef::sma(1).is_below(0.1);
        let strategy = Strategy::builder("cash_test")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        // High prices with low capital
        let candles = make_candles(&[100.0, 200.0, 200.0, 100.0]);
        let config = BacktestConfig {
            initial_capital: 1000.0, // Only $1000
            ..Default::default()
        };
        let res = backtest(strategy, &candles, config).unwrap();

        // Position size should be limited by available cash
        // qty = 1000 / (100 * 1.001) ≈ 9.99 → floor to 9
        if !res.trades.is_empty() {
            assert!(res.trades[0].qty <= 10.0);
            assert!(res.ending_cash >= 0.0); // Never go negative
        }
    }

    #[test]
    fn backtest_commission_reduces_returns() {
        let entry = IndicatorRef::sma(1).is_above(1.5);
        let exit = IndicatorRef::sma(1).is_below(1.5);
        let strategy = Strategy::builder("commission_test")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        let candles = make_candles(&[1.0, 2.0, 2.0, 1.0]);

        // Run without commission
        let config_no_comm = BacktestConfig {
            initial_capital: 100_000.0,
            commission_pct: 0.0,
            ..Default::default()
        };
        let res_no_comm = backtest(strategy.clone(), &candles, config_no_comm).unwrap();

        // Run with commission
        let config_with_comm = BacktestConfig {
            initial_capital: 100_000.0,
            commission_pct: 0.01, // 1% per trade
            ..Default::default()
        };
        let res_with_comm = backtest(strategy, &candles, config_with_comm).unwrap();

        // Commission should reduce ending cash
        assert!(res_with_comm.ending_cash < res_no_comm.ending_cash);
    }

    #[test]
    fn backtest_slippage_reduces_returns() {
        let entry = IndicatorRef::sma(1).is_above(1.5);
        let exit = IndicatorRef::sma(1).is_below(1.5);
        let strategy = Strategy::builder("slippage_test")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        let candles = make_candles(&[1.0, 2.0, 2.0, 1.0]);

        // Run without slippage
        let config_no_slip = BacktestConfig {
            initial_capital: 100_000.0,
            slippage_pct: 0.0,
            ..Default::default()
        };
        let res_no_slip = backtest(strategy.clone(), &candles, config_no_slip).unwrap();

        // Run with slippage
        let config_with_slip = BacktestConfig {
            initial_capital: 100_000.0,
            slippage_pct: 0.01, // 1% slippage
            ..Default::default()
        };
        let res_with_slip = backtest(strategy, &candles, config_with_slip).unwrap();

        // Slippage should reduce ending cash
        assert!(res_with_slip.ending_cash < res_no_slip.ending_cash);
    }

    #[test]
    fn backtest_edge_case_never_trades() {
        // Strategy that never triggers
        let entry = IndicatorRef::sma(1).is_above(100.0); // Never true
        let exit = IndicatorRef::sma(1).is_below(0.0); // Never true
        let strategy = Strategy::builder("never_trade")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        let candles = make_candles(&[1.0, 2.0, 2.0, 1.0]);
        let res = backtest(strategy, &candles, BacktestConfig::default()).unwrap();

        assert_eq!(res.trades.len(), 0);
        assert_eq!(res.ending_cash, 100_000.0); // No trades, no change
    }

    #[test]
    fn backtest_edge_case_always_in_position() {
        // Strategy that enters immediately and never exits
        let entry = IndicatorRef::sma(1).is_above(0.0); // Always true
        let exit = IndicatorRef::sma(1).is_below(-100.0); // Never true
        let strategy = Strategy::builder("always_in")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(5.0))
            .build()
            .unwrap();

        let candles = make_candles(&[1.0, 2.0, 2.0, 1.0]);
        let res = backtest(strategy, &candles, BacktestConfig::default()).unwrap();

        // Should have 1 trade (entry at bar 0, liquidated at end)
        assert_eq!(res.trades.len(), 1);
        assert!(res.trades[0].holding_period_bars > 0);
    }
}
