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
    /// Left is rising over N bars (current > N bars ago)
    IsRising(u32),
    /// Left is falling over N bars (current < N bars ago)
    IsFalling(u32),
}

/// Right-hand side of a comparison in a condition.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionGroup {
    /// All sub-conditions must be true
    AllOf(Vec<ConditionNode>),
    /// Any sub-condition must be true
    AnyOf(Vec<ConditionNode>),
}

/// A node in the condition tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionNode {
    Condition(Condition),
    Group(ConditionGroup),
}

/// Stop-loss configuration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TakeProfit {
    /// Fixed percentage above entry
    FixedPercent(f64),
    /// ATR multiple above entry
    AtrMultiple(f64),
}

/// Maximum nesting depth for condition groups (SPEC §5.3).
const MAX_NESTING_DEPTH: usize = 2;

/// Maximum conditions per group (SPEC §5.3).
const MAX_CONDITIONS_PER_GROUP: usize = 20;

/// A trading strategy composed of conditions and risk rules.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
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

    /// Build and validate the strategy (SPEC §5.3).
    pub fn build(self) -> crate::types::Result<Strategy> {
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

        if self.max_position_size_pct < 0.1 || self.max_position_size_pct > 100.0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_position_size_pct",
                value: self.max_position_size_pct.to_string(),
                reason: "must be between 0.1 and 100",
            });
        }

        if self.max_daily_loss_pct < 0.1 || self.max_daily_loss_pct > 50.0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_daily_loss_pct",
                value: self.max_daily_loss_pct.to_string(),
                reason: "must be between 0.1 and 50",
            });
        }

        if self.max_drawdown_pct < 1.0 || self.max_drawdown_pct > 100.0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_drawdown_pct",
                value: self.max_drawdown_pct.to_string(),
                reason: "must be between 1 and 100",
            });
        }

        if self.max_concurrent_positions == 0 {
            return Err(crate::types::MantisError::InvalidParameter {
                param: "max_concurrent_positions",
                value: "0".to_string(),
                reason: "must be at least 1",
            });
        }

        // Validate condition nesting depth and group sizes
        validate_condition_node(&entry, 0)?;
        if let Some(exit) = &self.exit {
            validate_condition_node(exit, 0)?;
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

/// Recursively validate condition nesting depth and group sizes (SPEC §5.3).
fn validate_condition_node(node: &ConditionNode, depth: usize) -> crate::types::Result<()> {
    if depth > MAX_NESTING_DEPTH {
        return Err(crate::types::MantisError::StrategyValidation(format!(
            "Condition nesting exceeds maximum depth of {}",
            MAX_NESTING_DEPTH
        )));
    }
    if let ConditionNode::Group(group) = node {
        let children = match group {
            ConditionGroup::AllOf(c) | ConditionGroup::AnyOf(c) => c,
        };
        if children.len() > MAX_CONDITIONS_PER_GROUP {
            return Err(crate::types::MantisError::StrategyValidation(format!(
                "Condition group exceeds maximum of {} conditions",
                MAX_CONDITIONS_PER_GROUP
            )));
        }
        for child in children {
            validate_condition_node(child, depth + 1)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_condition() -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            "sma20",
            Operator::CrossesAbove,
            CompareTarget::Value(100.0),
        ))
    }

    /// Helper to build a valid strategy with all mandatory fields.
    fn valid_builder() -> StrategyBuilder {
        Strategy::builder("test")
            .entry(sample_condition())
            .stop_loss(StopLoss::FixedPercent(2.0))
    }

    #[test]
    fn builder_requires_entry() {
        let result = Strategy::builder("test")
            .exit(sample_condition())
            .stop_loss(StopLoss::FixedPercent(2.0))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_requires_stop_loss() {
        let result = Strategy::builder("test").entry(sample_condition()).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_validates_position_size() {
        let result = valid_builder().max_position_size_pct(150.0).build();
        assert!(result.is_err());

        let result = valid_builder().max_position_size_pct(0.05).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_validates_daily_loss_bounds() {
        let result = valid_builder().max_daily_loss_pct(51.0).build();
        assert!(result.is_err());

        let result = valid_builder().max_daily_loss_pct(0.05).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_validates_drawdown_bounds() {
        let result = valid_builder().max_drawdown_pct(0.5).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_creates_valid_strategy() {
        let result = valid_builder().build();
        assert!(result.is_ok());
        let strategy = result.unwrap();
        assert_eq!(strategy.name, "test");
    }

    #[test]
    fn builder_rejects_excessive_nesting() {
        // depth 0: Group -> depth 1: Group -> depth 2: Group -> depth 3: Condition (too deep)
        let leaf = sample_condition();
        let depth2 = ConditionNode::Group(ConditionGroup::AllOf(vec![leaf]));
        let depth1 = ConditionNode::Group(ConditionGroup::AllOf(vec![depth2]));
        let depth0 = ConditionNode::Group(ConditionGroup::AllOf(vec![depth1]));

        let result = valid_builder().entry(depth0).build();
        assert!(result.is_err());
    }

    #[test]
    fn builder_accepts_valid_nesting() {
        // depth 0: Group -> depth 1: Group -> depth 2: Condition (within limit)
        let leaf = sample_condition();
        let depth1 = ConditionNode::Group(ConditionGroup::AllOf(vec![leaf]));
        let depth0 = ConditionNode::Group(ConditionGroup::AllOf(vec![depth1]));

        let result = valid_builder().entry(depth0).build();
        assert!(result.is_ok());
    }

    #[test]
    fn builder_rejects_oversized_group() {
        let conditions: Vec<ConditionNode> = (0..21).map(|_| sample_condition()).collect();
        let group = ConditionNode::Group(ConditionGroup::AllOf(conditions));

        let result = valid_builder().entry(group).build();
        assert!(result.is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn strategy_serde_round_trip() {
        let entry = ConditionNode::Condition(Condition::new(
            "sma_20",
            Operator::CrossesAbove,
            CompareTarget::Indicator("sma_50".to_string()),
        ));
        let exit = ConditionNode::Condition(Condition::new(
            "sma_20",
            Operator::CrossesBelow,
            CompareTarget::Indicator("sma_50".to_string()),
        ));
        let strategy = Strategy::builder("round_trip_test")
            .entry(entry)
            .exit(exit)
            .stop_loss(StopLoss::FixedPercent(2.0))
            .take_profit(TakeProfit::AtrMultiple(1.5))
            .max_concurrent_positions(3)
            .build()
            .unwrap();

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: Strategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn condition_group_nesting() {
        let cond1 = ConditionNode::Condition(Condition::new(
            "sma_20",
            Operator::IsAbove,
            CompareTarget::Value(100.0),
        ));
        let cond2 = ConditionNode::Condition(Condition::new(
            "rsi_14",
            Operator::IsBelow,
            CompareTarget::Value(70.0),
        ));
        let group = ConditionNode::Group(ConditionGroup::AllOf(vec![cond1, cond2]));
        assert!(matches!(group, ConditionNode::Group(_)));
    }
}
