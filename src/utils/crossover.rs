/// Detects cross-above/below events between two scalar series.
pub fn crosses_above(prev_left: f64, prev_right: f64, left: f64, right: f64) -> bool {
    left > right && prev_left <= prev_right
}

pub fn crosses_below(prev_left: f64, prev_right: f64, left: f64, right: f64) -> bool {
    left < right && prev_left >= prev_right
}
