/// Basic math helpers used across indicators.
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

pub fn sum(values: &[f64]) -> f64 {
    values.iter().sum()
}

pub fn variance(values: &[f64]) -> Option<f64> {
    let m = mean(values)?;
    Some(values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / values.len() as f64)
}

pub fn std_dev(values: &[f64]) -> Option<f64> {
    variance(values).map(|v| v.sqrt())
}
