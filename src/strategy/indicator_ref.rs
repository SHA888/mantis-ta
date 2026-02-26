use super::types::{CompareTarget, Condition, ConditionGroup, ConditionNode, Operator};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Reference to an indicator within a strategy, with convenience methods for building conditions.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
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
        Self::new(format!("macd_{}_{}_{}_line", fast, slow, signal))
    }

    /// MACD signal line convenience constructor.
    pub fn macd_signal(fast: usize, slow: usize, signal: usize) -> Self {
        Self::new(format!("macd_{}_{}_{}_signal", fast, slow, signal))
    }

    /// RSI convenience constructor.
    pub fn rsi(period: usize) -> Self {
        Self::new(format!("rsi{}", period))
    }

    /// Stochastic %K convenience constructor.
    pub fn stoch_k(k_period: usize, d_period: usize) -> Self {
        Self::new(format!("stoch_{}_{}_k", k_period, d_period))
    }

    /// Stochastic %D convenience constructor.
    pub fn stoch_d(k_period: usize, d_period: usize) -> Self {
        Self::new(format!("stoch_{}_{}_d", k_period, d_period))
    }

    /// Bollinger Bands upper convenience constructor.
    pub fn bb_upper(period: usize, std_dev: f64) -> Self {
        Self::new(format!("bb_{}_{}_upper", period, std_dev))
    }

    /// Bollinger Bands middle convenience constructor.
    pub fn bb_middle(period: usize, std_dev: f64) -> Self {
        Self::new(format!("bb_{}_{}_middle", period, std_dev))
    }

    /// Bollinger Bands lower convenience constructor.
    pub fn bb_lower(period: usize, std_dev: f64) -> Self {
        Self::new(format!("bb_{}_{}_lower", period, std_dev))
    }

    /// ATR convenience constructor.
    pub fn atr(period: usize) -> Self {
        Self::new(format!("atr{}", period))
    }

    /// Volume SMA convenience constructor.
    pub fn volume_sma(period: usize) -> Self {
        Self::new(format!("volume_sma_{}", period))
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

    /// Create a condition: this indicator equals a value (within epsilon).
    pub fn equals(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::Equals,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this indicator equals another indicator (within epsilon).
    pub fn equals_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::Equals,
            CompareTarget::Indicator(other.name),
        ))
    }

    /// Create a condition: this indicator is between two values.
    pub fn is_between(self, lower: f64, upper: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsBetween,
            CompareTarget::Range(lower, upper),
        ))
    }

    /// Create a condition: this indicator is rising over `bars` bars.
    pub fn is_rising(self, bars: u32) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsRising(bars),
            CompareTarget::None,
        ))
    }

    /// Create a condition: this indicator is falling over `bars` bars.
    pub fn is_falling(self, bars: u32) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            self.name,
            Operator::IsFalling(bars),
            CompareTarget::None,
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
#[derive(Debug, Clone, PartialEq)]
pub struct ScaledIndicatorRef {
    pub name: String,
    pub multiplier: f64,
}

impl ScaledIndicatorRef {
    /// Create a condition: this scaled indicator is above a value.
    pub fn is_above_value(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            format!("{}*{}", self.name, self.multiplier),
            Operator::IsAbove,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this scaled indicator is above another indicator.
    pub fn is_above_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            format!("{}*{}", self.name, self.multiplier),
            Operator::IsAbove,
            CompareTarget::Indicator(other.name),
        ))
    }

    /// Create a condition: this scaled indicator is below a value.
    pub fn is_below_value(self, value: f64) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            format!("{}*{}", self.name, self.multiplier),
            Operator::IsBelow,
            CompareTarget::Value(value),
        ))
    }

    /// Create a condition: this scaled indicator is below another indicator.
    pub fn is_below_indicator(self, other: IndicatorRef) -> ConditionNode {
        ConditionNode::Condition(Condition::new(
            format!("{}*{}", self.name, self.multiplier),
            Operator::IsBelow,
            CompareTarget::Indicator(other.name),
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

    #[test]
    fn scaled_is_above_indicator_has_correct_semantics() {
        // atr.scaled(2.0).is_above_indicator(price) should mean "atr*2 is above price"
        let cond = IndicatorRef::atr(14)
            .scaled(2.0)
            .is_above_indicator(IndicatorRef::new("price"));
        match cond {
            ConditionNode::Condition(c) => {
                assert_eq!(c.left, "atr14*2");
                assert_eq!(c.operator, Operator::IsAbove);
                assert_eq!(c.right, CompareTarget::Indicator("price".to_string()));
            }
            _ => panic!("expected Condition"),
        }
    }

    #[test]
    fn scaled_is_below_indicator_has_correct_semantics() {
        // atr.scaled(1.5).is_below_indicator(price) should mean "atr*1.5 is below price"
        let cond = IndicatorRef::atr(14)
            .scaled(1.5)
            .is_below_indicator(IndicatorRef::new("price"));
        match cond {
            ConditionNode::Condition(c) => {
                assert_eq!(c.left, "atr14*1.5");
                assert_eq!(c.operator, Operator::IsBelow);
                assert_eq!(c.right, CompareTarget::Indicator("price".to_string()));
            }
            _ => panic!("expected Condition"),
        }
    }

    #[test]
    fn scaled_is_above_value_has_correct_semantics() {
        let cond = IndicatorRef::atr(14).scaled(2.0).is_above_value(50.0);
        match cond {
            ConditionNode::Condition(c) => {
                assert_eq!(c.left, "atr14*2");
                assert_eq!(c.operator, Operator::IsAbove);
                assert_eq!(c.right, CompareTarget::Value(50.0));
            }
            _ => panic!("expected Condition"),
        }
    }
}
