/**
 * Performs a linear interpolation
 **/
pub fn lint(x: f64, x0: f64, x1: f64, y0: f64, y1: f64) -> f64 {
    let mut rx0 = x0;
    let mut rx1 = x1;
    if x0 > x1 {
        rx0 = x1;
        rx1 = x0;
    }

    if x < rx0 {
        y0
    } else if x > rx1 {
        y1
    } else {
        y0 + (y1 - y0) * (x - rx0) / (rx1 - rx0)
    }
}

/**
 * Clamp the input between min and max
 **/
pub fn clamp(input: f64, min: f64, max: f64) -> f64 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
