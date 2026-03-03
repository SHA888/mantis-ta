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
use crate::strategy::types::Strategy;
use crate::types::{Candle, ExitReason, MantisError, Result, Side, Signal};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionModel {
    NextBarOpen,
    CurrentBarClose,
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
            slippage_pct: 0.0005,
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
    pub trades: Vec<Trade>,
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

pub fn backtest(
    strategy: Strategy,
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

    let mut portfolio = Portfolio::new(config.initial_capital)?;
    let broker = BrokerSim::new();
    let mut engine = strategy_engine(strategy);
    let mut trades: Vec<Trade> = Vec::new();

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
    }

    Ok(BacktestResult {
        starting_cash: config.initial_capital,
        ending_cash: portfolio.cash(),
        trades,
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
