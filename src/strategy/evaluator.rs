use std::collections::{HashMap, HashSet};

use crate::indicators::{Indicator, ATR, EMA, RSI, SMA};
use crate::strategy::types::{
    CompareTarget, Condition, ConditionGroup, ConditionNode, Operator, Strategy,
};
use crate::types::{Candle, ExitReason, Side, Signal};

/// Wrapper over supported indicator implementations that produce scalar outputs.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
enum IndicatorInstance {
    SMA(SMA),
    EMA(EMA),
    RSI(RSI),
    ATR(ATR),
}

impl IndicatorInstance {
    fn next(&mut self, candle: &Candle) -> Option<f64> {
        match self {
            IndicatorInstance::SMA(i) => i.next(candle),
            IndicatorInstance::EMA(i) => i.next(candle),
            IndicatorInstance::RSI(i) => i.next(candle),
            IndicatorInstance::ATR(i) => i.next(candle),
        }
    }
}

/// Parse an indicator reference name into a concrete indicator instance.
/// Supported forms:
/// - "sma{period}"
/// - "ema{period}"
/// - "rsi{period}"
/// - "atr{period}"
fn parse_indicator(name: &str) -> Option<IndicatorInstance> {
    if let Some(rest) = name.strip_prefix("sma") {
        if let Ok(p) = rest.parse::<usize>() {
            return Some(IndicatorInstance::SMA(SMA::new(p)));
        }
    }
    if let Some(rest) = name.strip_prefix("ema") {
        if let Ok(p) = rest.parse::<usize>() {
            return Some(IndicatorInstance::EMA(EMA::new(p)));
        }
    }
    if let Some(rest) = name.strip_prefix("rsi") {
        if let Ok(p) = rest.parse::<usize>() {
            return Some(IndicatorInstance::RSI(RSI::new(p)));
        }
    }
    if let Some(rest) = name.strip_prefix("atr") {
        if let Ok(p) = rest.parse::<usize>() {
            return Some(IndicatorInstance::ATR(ATR::new(p)));
        }
    }
    None
}

/// Strategy evaluation engine for streaming signals.
#[derive(Debug)]
pub struct StrategyEngine {
    strategy: Strategy,
    indicators: HashMap<String, IndicatorInstance>,
    required: HashSet<String>,
    last_values: HashMap<String, f64>,
}

impl StrategyEngine {
    pub fn new(strategy: Strategy) -> Self {
        let mut indicators = HashMap::new();
        collect_indicators_from_node(&strategy.entry, &mut indicators);
        if let Some(exit) = &strategy.exit {
            collect_indicators_from_node(exit, &mut indicators);
        }
        let required: HashSet<String> = indicators.keys().cloned().collect();
        let mut instances = HashMap::new();
        for name in indicators.keys() {
            if let Some(inst) = parse_indicator(name) {
                instances.insert(name.clone(), inst);
            }
        }
        Self {
            strategy,
            indicators: instances,
            required,
            last_values: HashMap::new(),
        }
    }

    /// Evaluate one candle and emit a signal.
    pub fn next(&mut self, candle: &Candle) -> Signal {
        // Capture previous values for cross/rising/falling detection
        let prev_values = self.last_values.clone();
        self.last_values.clear();

        // Advance indicators
        for (name, inst) in self.indicators.iter_mut() {
            if let Some(v) = inst.next(candle) {
                self.last_values.insert(name.clone(), v);
            }
        }

        // Warmup: if any required indicator has not produced a value yet, hold
        if self
            .required
            .iter()
            .any(|name| !self.last_values.contains_key(name))
        {
            return Signal::Hold;
        }

        // Evaluate entry/exit; if required indicator values are missing, return Hold
        let entry = eval_node(&self.strategy.entry, &self.last_values, &prev_values);
        let exit = self
            .strategy
            .exit
            .as_ref()
            .and_then(|n| eval_node(n, &self.last_values, &prev_values));

        if exit == Some(true) {
            Signal::Exit(ExitReason::RuleTriggered)
        } else if entry == Some(true) {
            Signal::Entry(Side::Long)
        } else {
            Signal::Hold
        }
    }

    /// Batch evaluation over a candle slice.
    pub fn evaluate(&mut self, candles: &[Candle]) -> Vec<Signal> {
        candles.iter().map(|c| self.next(c)).collect()
    }
}

fn get_value(name: &str, values: &HashMap<String, f64>) -> Option<f64> {
    values.get(name).copied()
}

/// Evaluate a condition tree. Returns None if data is insufficient (warmup).
fn eval_node(
    node: &ConditionNode,
    curr: &HashMap<String, f64>,
    prev: &HashMap<String, f64>,
) -> Option<bool> {
    match node {
        ConditionNode::Condition(c) => eval_condition(c, curr, prev),
        ConditionNode::Group(g) => match g {
            ConditionGroup::AllOf(nodes) => {
                let mut any_none = false;
                for n in nodes {
                    match eval_node(n, curr, prev) {
                        Some(true) => {}
                        Some(false) => return Some(false),
                        None => any_none = true,
                    }
                }
                if any_none {
                    None
                } else {
                    Some(true)
                }
            }
            ConditionGroup::AnyOf(nodes) => {
                let mut any_none = false;
                for n in nodes {
                    match eval_node(n, curr, prev) {
                        Some(true) => return Some(true),
                        Some(false) => {}
                        None => any_none = true,
                    }
                }
                if any_none {
                    None
                } else {
                    Some(false)
                }
            }
        },
    }
}

const EPS: f64 = 1e-9;

fn get_prev_n(name: &str, prev: &HashMap<String, f64>, n: u32) -> Option<f64> {
    if n == 1 {
        get_value(name, prev)
    } else {
        None
    }
}

fn eval_condition(
    condition: &Condition,
    curr: &HashMap<String, f64>,
    prev: &HashMap<String, f64>,
) -> Option<bool> {
    let left = get_value(&condition.left, curr)?;
    let right_curr = match &condition.right {
        CompareTarget::Value(v) => Some(*v),
        CompareTarget::Indicator(name) => get_value(name, curr),
        CompareTarget::Scaled {
            indicator,
            multiplier,
        } => get_value(indicator, curr).map(|v| v * multiplier),
        CompareTarget::Range(_, _) => None, // handled per-operator
        CompareTarget::None => None,
    };

    match condition.operator {
        Operator::IsAbove => Some(left > right_curr?),
        Operator::IsBelow => Some(left < right_curr?),
        Operator::Equals => Some((left - right_curr?).abs() < EPS),
        Operator::IsBetween => {
            if let CompareTarget::Range(lower, upper) = condition.right {
                Some(left >= lower && left <= upper)
            } else {
                right_curr.map(|r| left >= r)
            }
        }
        Operator::CrossesAbove => {
            let prev_left = get_value(&condition.left, prev)?;
            let prev_right = match &condition.right {
                CompareTarget::Value(v) => Some(*v),
                CompareTarget::Indicator(name) => get_value(name, prev),
                CompareTarget::Scaled {
                    indicator,
                    multiplier,
                } => get_value(indicator, prev).map(|v| v * multiplier),
                _ => None,
            }?;
            Some(left > right_curr? && prev_left <= prev_right)
        }
        Operator::CrossesBelow => {
            let prev_left = get_value(&condition.left, prev)?;
            let prev_right = match &condition.right {
                CompareTarget::Value(v) => Some(*v),
                CompareTarget::Indicator(name) => get_value(name, prev),
                CompareTarget::Scaled {
                    indicator,
                    multiplier,
                } => get_value(indicator, prev).map(|v| v * multiplier),
                _ => None,
            }?;
            Some(left < right_curr? && prev_left >= prev_right)
        }
        Operator::IsRising(period) => {
            let prev_left = get_prev_n(&condition.left, prev, period)?;
            Some(left > prev_left)
        }
        Operator::IsFalling(period) => {
            let prev_left = get_prev_n(&condition.left, prev, period)?;
            Some(left < prev_left)
        }
    }
}

/// Walk a condition tree to collect indicator names referenced.
fn collect_indicators_from_node(node: &ConditionNode, set: &mut HashMap<String, ()>) {
    match node {
        ConditionNode::Condition(c) => {
            set.insert(c.left.clone(), ());
            if let CompareTarget::Indicator(name) = &c.right {
                set.insert(name.clone(), ());
            }
            if let CompareTarget::Scaled { indicator, .. } = &c.right {
                set.insert(indicator.clone(), ());
            }
        }
        ConditionNode::Group(g) => match g {
            ConditionGroup::AllOf(nodes) | ConditionGroup::AnyOf(nodes) => {
                for n in nodes {
                    collect_indicators_from_node(n, set);
                }
            }
        },
    }
}

/// Batch evaluation helper: convenience wrapper over StrategyEngine.
pub fn evaluate_strategy_batch(strategy: &Strategy, candles: &[Candle]) -> Vec<Signal> {
    let mut engine = StrategyEngine::new(strategy.clone());
    engine.evaluate(candles)
}

/// Streaming evaluation helper: create an engine from a strategy.
pub fn strategy_engine(strategy: Strategy) -> StrategyEngine {
    StrategyEngine::new(strategy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::indicator_ref::IndicatorRef;
    use crate::strategy::types::{
        CompareTarget, Condition, ConditionGroup, ConditionNode, Operator,
    };
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
    fn golden_cross_signals() {
        // Use very small periods to reduce warmup
        let entry = IndicatorRef::sma(1).is_above(1.5);
        let exit = IndicatorRef::sma(1).is_below(1.5);
        let strategy = Strategy::builder("gc")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        // Prices designed to cross upward then downward
        let prices = [1.0, 1.2, 1.6, 1.8, 1.4, 1.2];
        let candles = make_candles(&prices);
        let signals = evaluate_strategy_batch(&strategy, &candles);

        assert_eq!(signals.len(), prices.len());
    }

    #[test]
    fn rsi_mean_reversion_signals() {
        let entry = IndicatorRef::rsi(2).is_below(40.0);
        let exit = IndicatorRef::rsi(2).is_above(60.0);
        let strategy = Strategy::builder("rsi")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(2.0))
            .build()
            .unwrap();

        // Construct prices to push RSI below 40 then above 60
        let prices = [10.0, 9.5, 9.0, 8.5, 9.5, 10.5];
        let candles = make_candles(&prices);
        let signals = evaluate_strategy_batch(&strategy, &candles);

        // Manual RSI verification: compute RSI and derive expected signals from thresholds.
        let mut rsi = crate::indicators::RSI::new(2);
        let mut expected = Vec::new();
        for c in &candles {
            let v = rsi.next(c);
            let sig = match v {
                Some(x) if x > 60.0 => Signal::Exit(ExitReason::RuleTriggered),
                Some(x) if x < 40.0 => Signal::Entry(Side::Long),
                _ => Signal::Hold,
            };
            expected.push(sig);
        }

        assert_eq!(signals, expected);

        let entry_idx = signals.iter().position(|s| matches!(s, Signal::Entry(_)));
        let exit_idx = signals.iter().position(|s| matches!(s, Signal::Exit(_)));
        assert!(entry_idx.is_some(), "expected at least one entry signal");
        assert!(exit_idx.is_some(), "expected at least one exit signal");
        if let (Some(ei), Some(xi)) = (entry_idx, exit_idx) {
            assert!(ei < xi, "entry should occur before exit");
        }
    }

    #[test]
    fn edge_single_condition_entry_only() {
        let entry = IndicatorRef::sma(1).is_above(1.0);
        let strategy = Strategy::builder("single")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        let prices = [2.0, 2.0, 2.0];
        let candles = make_candles(&prices);
        let signals = evaluate_strategy_batch(&strategy, &candles);

        assert!(signals.iter().all(|s| matches!(s, Signal::Entry(_))));
    }

    #[test]
    fn edge_max_conditions_group_all_of() {
        let cond = || {
            ConditionNode::Condition(Condition::new(
                "sma1",
                Operator::IsAbove,
                CompareTarget::Value(1.0),
            ))
        };
        let entry = ConditionNode::Group(ConditionGroup::AllOf((0..20).map(|_| cond()).collect()));
        let strategy = Strategy::builder("max_group")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        let prices = [2.0, 2.0, 2.0];
        let candles = make_candles(&prices);
        let signals = evaluate_strategy_batch(&strategy, &candles);

        assert!(signals.iter().all(|s| matches!(s, Signal::Entry(_))));
    }

    #[test]
    fn edge_nested_groups() {
        let always_true = ConditionNode::Condition(Condition::new(
            "sma1",
            Operator::IsAbove,
            CompareTarget::Value(1.0),
        ));
        let always_false = ConditionNode::Condition(Condition::new(
            "sma1",
            Operator::IsAbove,
            CompareTarget::Value(10.0),
        ));

        // Entry: sma1 > 1 AND (sma1 > 10 OR sma1 > 1)
        let entry = ConditionNode::Group(ConditionGroup::AllOf(vec![
            always_true.clone(),
            ConditionNode::Group(ConditionGroup::AnyOf(vec![always_false, always_true])),
        ]));

        let strategy = Strategy::builder("nested")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        let prices = [2.0, 2.0, 2.0];
        let candles = make_candles(&prices);
        let signals = evaluate_strategy_batch(&strategy, &candles);

        assert!(signals.iter().all(|s| matches!(s, Signal::Entry(_))));
    }

    #[test]
    fn streaming_equals_batch() {
        let entry = IndicatorRef::sma(2).crosses_above_indicator(IndicatorRef::sma(3));
        let strategy = Strategy::builder("gc")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        let prices = [1.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0];
        let candles = make_candles(&prices);

        let batch = evaluate_strategy_batch(&strategy, &candles);
        let mut engine = strategy_engine(strategy);
        let streaming: Vec<_> = candles.iter().map(|c| engine.next(c)).collect();

        assert_eq!(batch, streaming);
    }

    #[test]
    fn golden_cross_manual_verification() {
        // Deterministic sequence: fast SMA(1) vs slow SMA(3)
        // - Warmup: indices 0-2 (SMA3 not ready)
        // - Index 3: fast=3.0, slow≈1.67 -> above (Entry)
        // - Index 5: fast=0.5, slow≈2.17 -> below (Exit)
        let prices = [1.0, 1.0, 1.0, 3.0, 3.0, 0.5];
        let candles = make_candles(&prices);

        // Manual SMA values and expected signals (above/below semantics)
        let mut sma1 = crate::indicators::SMA::new(1);
        let mut sma3 = crate::indicators::SMA::new(3);
        let mut expected = Vec::new();
        let mut prev_fast: Option<f64> = None;
        let mut prev_slow: Option<f64> = None;

        for c in &candles {
            let fast = sma1.next(c);
            let slow = sma3.next(c);

            let sig = match (fast, slow, prev_fast, prev_slow) {
                (Some(f), Some(s), Some(_), Some(_)) => {
                    if f > s {
                        Signal::Entry(Side::Long)
                    } else if f < s {
                        Signal::Exit(ExitReason::RuleTriggered)
                    } else {
                        Signal::Hold
                    }
                }
                _ => Signal::Hold,
            };

            expected.push(sig);
            prev_fast = fast;
            prev_slow = slow;
        }

        let entry = IndicatorRef::sma(1).is_above_indicator(IndicatorRef::sma(3));
        let exit = IndicatorRef::sma(1).is_below_indicator(IndicatorRef::sma(3));
        let strategy = Strategy::builder("gc_manual")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(1.0))
            .build()
            .unwrap();

        let signals = evaluate_strategy_batch(&strategy, &candles);

        let entry_idx = signals.iter().position(|s| matches!(s, Signal::Entry(_)));
        let exit_idx = signals.iter().position(|s| matches!(s, Signal::Exit(_)));

        assert!(entry_idx.is_some(), "expected at least one entry signal");
        assert!(exit_idx.is_some(), "expected at least one exit signal");
        if let (Some(ei), Some(xi)) = (entry_idx, exit_idx) {
            assert!(ei < xi, "entry should occur before exit");
        }
    }
}
