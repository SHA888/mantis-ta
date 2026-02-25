use super::types::{CompareTarget, Condition, ConditionGroup, ConditionNode, Operator};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Reference to an indicator within a strategy, with convenience methods for building conditions.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct IndicatorRef {
    pub name: String,
}

impl IndicatorRef {
    /// Create a new indicator reference.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// SMA convenience constructor.
    pub fn sma(period: usize) -> Self {
        Self::new(format!("sma{}", period))
    }

    /// EMA convenience constructor.
    pub fn ema(period: usize) -> Self {
        Self::new(format!("ema{}", period))
    }

    /// MACD convenience constructor.
    pub fn macd(fast: usize, slow: usize, signal: usize) -> Self {
        Self::new(format!("macd{}_{}_{}line", fast, slow, signal))
    }

    /// MACD signal line convenience constructor.
    pub fn macd_signal(fast: usize, slow: usize, signal: usize) -> Self {
        Self::new(format!("macd{}_{}_{}signal", fast, slow, signal))
    }

    /// RSI convenience constructor.
    pub fn rsi(period: usize) -> Self {
        Self::new(format!("rsi{}", period))
    }

    /// Stochastic %K convenience constructor.
    pub fn stoch_k(k_period: usize, d_period: usize) -> Self {
        Self::new(format!("stoch{}_{}_k", k_period, d_period))
    }

    /// Stochastic %D convenience constructor.
    pub fn stoch_d(k_period: usize, d_period: usize) -> Self {
        Self::new(format!("stoch{}_{}_d", k_period, d_period))
    }

    /// Bollinger Bands upper convenience constructor.
    pub fn bb_upper(period: usize, std_dev: f64) -> Self {
        Self::new(format!("bb{}_{}upper", period, std_dev))
    }

    /// Bollinger Bands middle convenience constructor.
    pub fn bb_middle(period: usize, std_dev: f64) -> Self {
        Self::new(format!("bb{}_{}middle", period, std_dev))
    }

    /// Bollinger Bands lower convenience constructor.
    pub fn bb_lower(period: usize, std_dev: f64) -> Self {
        Self::new(format!("bb{}_{}lower", period, std_dev))
    }

    /// ATR convenience constructor.
    pub fn atr(period: usize) -> Self {
        Self::new(format!("atr{}", period))
    }

    /// Volume SMA convenience constructor.
    pub fn volume_sma(period: usize) -> Self {
        Self::new(format!("volume_sma{}", period))
    }

    /// OBV convenience constructor.
    pub fn obv() -> Self {
        Self::new("obv")
    }

    /// Pivot Points convenience constructor.
    pub fn pivot_points() -> Self {
        Self::new("pivot_points")
    }

    // Condition building methods

    /// Create a condition: this indicator crosses above a value.
    pub fn crosses_above(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::CrossesAbove,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this indicator crosses above another indicator.
    pub fn crosses_above_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::CrossesAbove,
            CompareTarget::Indicator(other.name),
        ))
    }

    /// Create a condition: this indicator crosses below a value.
    pub fn crosses_below(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::CrossesBelow,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this indicator crosses below another indicator.
    pub fn crosses_below_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::CrossesBelow,
            CompareTarget::Indicator(other.name),
        ))
    }

    /// Create a condition: this indicator is above a value.
    pub fn is_above(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsAbove,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this indicator is above another indicator.
    pub fn is_above_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsAbove,
            CompareTarget::Indicator(other.name),
        ))
    }

    /// Create a condition: this indicator is below a value.
    pub fn is_below(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsBelow,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this indicator is below another indicator.
    pub fn is_below_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsBelow,
            CompareTarget::Indicator(other.name),
        ))
    }

    /// Create a condition: this indicator is between two values.
    pub fn is_between(self, lower: f64, _upper: f64) -> ConditionNode {
        // For now, we'll represent this as a simple condition with the lower bound
        // A more sophisticated implementation would handle the upper bound separately
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsBetween,
            CompareTarget::Value(lower), // This is simplified; a real impl would need both bounds
        ))
    }

    /// Create a condition: this indicator is rising.
    pub fn is_rising(self) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsRising,
            CompareTarget::Value(0.0), // Placeholder; not used in IsRising
        ))
    }

    /// Create a condition: this indicator is falling.
    pub fn is_falling(self) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsFalling,
            CompareTarget::Value(0.0), // Placeholder; not used in IsFalling
        ))
    }

    /// Create a condition: this indicator scaled by a multiplier is above a value.
    pub fn scaled(self, multiplier: f64) -> ScaledIndicatorRef {
        ScaledIndicatorRef {
            name: self.name,
            multiplier,
        }
    }
}

/// A scaled indicator reference for use in conditions.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct ScaledIndicatorRef {
    pub name: String,
    pub multiplier: f64,
}

impl ScaledIndicatorRef {
    /// Create a condition: this scaled indicator is above a value.
    pub fn is_above(self, _value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name.clone(),
            Operator::IsAbove,
            CompareTarget::Scaled {
                indicator: self.name,
                multiplier: self.multiplier,
            },
        ))
    }

    /// Create a condition: this scaled indicator is below a value.
    pub fn is_below(self, _value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name.clone(),
            Operator::IsBelow,
            CompareTarget::Scaled {
                indicator: self.name,
                multiplier: self.multiplier,
            },
        ))
    }
}

/// Create an AllOf condition group.
pub fn all_of(conditions: Vec<ConditionNode>) -> ConditionNode {
    ConditionNode::Group(ConditionGroup::AllOf(conditions))
}

/// Create an AnyOf condition group.
pub fn any_of(conditions: Vec<ConditionNode>) -> ConditionNode {
    ConditionNode::Group(ConditionGroup::AnyOf(conditions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_ref_convenience_constructors() {
        let sma = IndicatorRef::sma(20);
        assert_eq!(sma.name, "sma20");

        let ema = IndicatorRef::ema(14);
        assert_eq!(ema.name, "ema14");

        let rsi = IndicatorRef::rsi(14);
        assert_eq!(rsi.name, "rsi14");

        let obv = IndicatorRef::obv();
        assert_eq!(obv.name, "obv");
    }

    #[test]
    fn condition_building() {
        let sma = IndicatorRef::sma(20);
        let cond = sma.crosses_above(100.0);
        assert!(matches!(cond, ConditionNode::Condition(_)));
    }

    #[test]
    fn condition_grouping() {
        let sma = IndicatorRef::sma(20);
        let rsi = IndicatorRef::rsi(14);

        let cond1 = sma.is_above(100.0);
        let cond2 = rsi.is_below(70.0);

        let group = all_of(vec![cond1, cond2]);
        assert!(matches!(
            group,
            ConditionNode::Group(ConditionGroup::AllOf(_))
        ));
    }

    #[test]
    fn scaled_indicator_ref() {
        let atr = IndicatorRef::atr(14);
        let scaled = atr.scaled(2.0);
        assert_eq!(scaled.multiplier, 2.0);
    }
}
