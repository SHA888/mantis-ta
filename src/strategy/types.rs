#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Comparison operators for condition evaluation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Left crosses above right
    CrossesAbove,
    /// Left crosses below right
    CrossesBelow,
    /// Left is strictly above right
    IsAbove,
    /// Left is strictly below right
    IsBelow,
    /// Left is between lower and upper bounds
    IsBetween,
    /// Left equals right (within epsilon)
    Equals,
    /// Left is rising (current > previous)
    IsRising,
    /// Left is falling (current < previous)
    IsFalling,
}

/// Right-hand side of a comparison in a condition.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum CompareTarget {
    /// Compare against a fixed scalar value
    Value(f64),
    /// Compare against another indicator's output
    Indicator(String),
    /// Compare against a scaled value (e.g., ATR * 2.0)
    Scaled { indicator: String, multiplier: f64 },
    /// Compare against a range of values (lower, upper)
    Range(f64, f64),
    /// No compare target (used for unary operators like IsRising/IsFalling)
    None,
}

/// A single condition: left indicator, operator, right target.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Condition {
    pub left: String, // indicator name/id
    pub operator: Operator,
    pub right: CompareTarget,
}

impl Condition {
    pub fn new(left: impl Into<String>, operator: Operator, right: CompareTarget) -> Self {
        Self {
            left: left.into(),
            operator,
            right,
        }
    }
}

/// Logical grouping of conditions.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ConditionGroup {
    /// All sub-conditions must be true
    AllOf(Vec<ConditionNode>),
    /// Any sub-condition must be true
    AnyOf(Vec<ConditionNode>),
}

/// A node in the condition tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ConditionNode {
    Condition(Condition),
    Group(ConditionGroup),
}

/// Stop-loss configuration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum StopLoss {
    /// Fixed percentage below entry
    FixedPercent(f64),
    /// ATR multiple below entry
    AtrMultiple(f64),
    /// Trailing stop: fixed percentage below highest price
    Trailing(f64),
}

/// Take-profit configuration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum TakeProfit {
    /// Fixed percentage above entry
    FixedPercent(f64),
    /// ATR multiple above entry
    AtrMultiple(f64),
}

/// A trading strategy composed of conditions and risk rules.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub timeframe: crate::types::Timeframe,
    pub entry: ConditionNode,
    pub exit: Option<ConditionNode>,
    pub stop_loss: StopLoss,
    pub take_profit: Option<TakeProfit>,
    pub max_position_size_pct: f64,
    pub max_daily_loss_pct: f64,
    pub max_drawdown_pct: f64,
    pub max_concurrent_positions: usize,
}

impl Strategy {
    /// Create a new strategy builder.
    pub fn builder(name: impl Into<String>) -> StrategyBuilder {
        StrategyBuilder {
            name: name.into(),
            timeframe: crate::types::Timeframe::D1,
            entry: None,
            exit: None,
            stop_loss: None,
            take_profit: None,
            max_position_size_pct: 5.0,
            max_daily_loss_pct: 2.0,
            max_drawdown_pct: 10.0,
            max_concurrent_positions: 1,
        }
    }
}

/// Fluent builder for constructing strategies with validation.
#[derive(Debug)]
pub struct StrategyBuilder {
    name: String,
    timeframe: crate::types::Timeframe,
    entry: Option<ConditionNode>,
    exit: Option<ConditionNode>,
    stop_loss: Option<StopLoss>,
    take_profit: Option<TakeProfit>,
    max_position_size_pct: f64,
    max_daily_loss_pct: f64,
    max_drawdown_pct: f64,
    max_concurrent_positions: usize,
}

impl StrategyBuilder {
    pub fn timeframe(mut self, tf: crate::types::Timeframe) -> Self {
        self.timeframe = tf;
        self
    }

    pub fn entry(mut self, condition: ConditionNode) -> Self {
        self.entry = Some(condition);
        self
    }

    pub fn exit(mut self, condition: ConditionNode) -> Self {
        self.exit = Some(condition);
        self
    }

    pub fn stop_loss(mut self, sl: StopLoss) -> Self {
        self.stop_loss = Some(sl);
        self
    }

    pub fn take_profit(mut self, tp: TakeProfit) -> Self {
        self.take_profit = Some(tp);
        self
    }

    pub fn max_position_size_pct(mut self, pct: f64) -> Self {
        self.max_position_size_pct = pct;
        self
    }

    pub fn max_daily_loss_pct(mut self, pct: f64) -> Self {
        self.max_daily_loss_pct = pct;
        self
    }

    pub fn max_drawdown_pct(mut self, pct: f64) -> Self {
        self.max_drawdown_pct = pct;
        self
    }

    pub fn max_concurrent_positions(mut self, count: usize) -> Self {
        self.max_concurrent_positions = count;
        self
    }

    /// Build and validate the strategy.
    pub fn build(self) -> crate::types::Result<Strategy> {
        // Validation rules from SPEC §5.3
        let Some(entry) = self.entry else {
            return Err(crate::types::MantisError::StrategyValidation(
                "Strategy must have an entry condition".to_string(),
            ));
        };

        let Some(stop_loss) = self.stop_loss else {
            return Err(crate::types::MantisError::StrategyValidation(
                "Strategy must have a stop-loss rule".to_string(),
            ));
        };

        if self.max_position_size_pct <= 0.0 || self.max_position_size_pct > 100.0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_position_size_pct",
                value: self.max_position_size_pct.to_string(),
                reason: "must be between 0 and 100",
            });
        }

        if self.max_daily_loss_pct <= 0.0 || self.max_daily_loss_pct > 100.0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_daily_loss_pct",
                value: self.max_daily_loss_pct.to_string(),
                reason: "must be between 0 and 100",
            });
        }

        if self.max_drawdown_pct <= 0.0 || self.max_drawdown_pct > 100.0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_drawdown_pct",
                value: self.max_drawdown_pct.to_string(),
                reason: "must be between 0 and 100",
            });
        }

        if self.max_concurrent_positions == 0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_concurrent_positions",
                value: "0".to_string(),
                reason: "must be at least 1",
            });
        }

        Ok(Strategy {
            name: self.name,
            timeframe: self.timeframe,
            entry,
            exit: self.exit,
            stop_loss,
            take_profit: self.take_profit,
            max_position_size_pct: self.max_position_size_pct,
            max_daily_loss_pct: self.max_daily_loss_pct,
            max_drawdown_pct: self.max_drawdown_pct,
            max_concurrent_positions: self.max_concurrent_positions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_entry() {
        let result = Strategy::builder("test")
            .stop_loss(StopLoss::FixedPercent(2.0))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_requires_stop_loss() {
        let entry = ConditionNode::Condition(Condition::new(
            "sma20",
            Operator::CrossesAbove,
            CompareTarget::Value(100.0),
        ));
        let result = Strategy::builder("test").entry(entry).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_validates_position_size() {
        let entry = ConditionNode::Condition(Condition::new(
            "sma20",
            Operator::CrossesAbove,
            CompareTarget::Value(100.0),
        ));
        let result = Strategy::builder("test")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(2.0))
            .max_position_size_pct(150.0)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_creates_valid_strategy() {
        let entry = ConditionNode::Condition(Condition::new(
            "sma20",
            Operator::CrossesAbove,
            CompareTarget::Value(100.0),
        ));
        let result = Strategy::builder("golden_cross")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(2.0))
            .take_profit(TakeProfit::FixedPercent(5.0))
            .build();
        assert!(result.is_ok());
        let strategy = result.unwrap();
        assert_eq!(strategy.name, "golden_cross");
        assert!(strategy.take_profit.is_some());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn strategy_serde_round_trip() {
        let entry = ConditionNode::Condition(Condition::new(
            "sma_20",
            Operator::CrossesAbove,
            CompareTarget::Indicator("sma_50".to_string()),
        ));
        let strategy = Strategy::builder("round_trip_test")
            .entry(entry)
            .stop_loss(StopLoss::FixedPercent(2.0))
            .take_profit(TakeProfit::AtrMultiple(1.5))
            .max_concurrent_positions(3)
            .build()
            .unwrap();

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: Strategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy.name, deserialized.name);
        assert_eq!(
            strategy.max_concurrent_positions,
            deserialized.max_concurrent_positions
        );
        assert_eq!(
            strategy.max_position_size_pct,
            deserialized.max_position_size_pct
        );
    }

    #[test]
    fn condition_group_nesting() {
        let cond1 = ConditionNode::Condition(Condition::new(
            "sma20",
            Operator::IsAbove,
            CompareTarget::Value(100.0),
        ));
        let cond2 = ConditionNode::Condition(Condition::new(
            "rsi14",
            Operator::IsBelow,
            CompareTarget::Value(70.0),
        ));
        let group = ConditionNode::Group(ConditionGroup::AllOf(vec![cond1, cond2]));
        assert!(matches!(group, ConditionNode::Group(_)));
    }
}
