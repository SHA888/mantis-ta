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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionModel {
    NextBarOpen,
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
            eprintln!("Warning: equity curve contains non-finite value at index {}", i);
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub commission_per_trade: f64,
    pub commission_pct: f64,
    pub slippage_pct: f64,
    pub execution: ExecutionModel,
    pub fractional_shares: bool,
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub entry_timestamp: i64,
    pub exit_timestamp: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub qty: f64,
    pub pnl: f64,
    pub exit_reason: ExitReason,
    pub holding_period_bars: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct BacktestResult {
    pub starting_cash: f64,
    pub ending_cash: f64,
    pub equity_curve: Vec<(i64, f64)>,
    pub metrics: BacktestMetrics,
    pub trades: Vec<Trade>,
    pub warnings: Vec<String>,
    pub sensitivity: Vec<ParameterSensitivity>,
    pub walk_forward: Option<WalkForwardResult>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BacktestMetrics {
    pub total_return: f64,
    pub cagr: Option<f64>,
    pub annualized_vol: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub win_rate: Option<f64>,
    pub profit_factor: Option<f64>,
    pub average_win: Option<f64>,
    pub average_loss: Option<f64>,
    pub total_trades: usize,
    pub exposure_ratio: Option<f64>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParameterSensitivity {
    pub factor: f64,
    pub commission_pct: f64,
    pub slippage_pct: f64,
    pub metrics: BacktestMetrics,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WalkForwardResult {
    pub split_index: usize,
    pub train_metrics: BacktestMetrics,
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
    if condition_count > 7 {
        warnings.push(format!(
            "Excessive condition warning: {} conditions (> 7) — potential overfitting",
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
    let base = backtest_once(&strategy, candles, config)?;

    // Parameter sensitivity: scale commission_pct and slippage_pct by factors
    let factors = [0.9, 1.0, 1.1];
    let mut sensitivity = Vec::new();
    for factor in factors {
        let mut cfg = config;
        cfg.commission_pct *= factor;
        cfg.slippage_pct *= factor;
        if let Ok(res) = backtest_once(&strategy, candles, cfg) {
            sensitivity.push(ParameterSensitivity {
                factor,
                commission_pct: cfg.commission_pct,
                slippage_pct: cfg.slippage_pct,
                metrics: res.metrics,
            });
        }
    }

    // Walk-forward: 70/30 split (time-ordered)
    let split_index = ((candles.len() as f64) * 0.7).floor() as usize;
    let walk_forward = if split_index >= 2 && split_index + 2 <= candles.len() {
        let train = &candles[..split_index];
        let test = &candles[split_index..];
        match (backtest_once(&strategy, train, config), backtest_once(&strategy, test, config)) {
            (Ok(train_res), Ok(test_res)) => Some(WalkForwardResult {
                split_index,
                train_metrics: train_res.metrics,
                test_metrics: test_res.metrics,
            }),
            _ => None,
        }
    } else {
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
    use crate::strategy::indicator_ref::IndicatorRef;
    use crate::strategy::StopLoss;

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
}
