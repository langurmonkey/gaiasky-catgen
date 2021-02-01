pub fn parse_i32(val_str: Option<&&str>) -> i32 {
    match val_str {
        Some(val) => val.to_string().parse::<i32>().unwrap(),
        None => 0,
    }
}

pub fn parse_i64(val_str: Option<&&str>) -> i64 {
    match val_str {
        Some(val) => val.to_string().parse::<i64>().unwrap(),
        None => 0,
    }
}

pub fn parse_f32(val_str: Option<&&str>) -> f32 {
    match val_str {
        Some(val) => val.to_string().parse::<f32>().map_or(f32::NAN, |v| v),
        None => f32::NAN,
    }
}

pub fn parse_f64(val_str: Option<&&str>) -> f64 {
    match val_str {
        Some(val) => val.to_string().parse::<f64>().map_or(f64::NAN, |v| v),
        None => f64::NAN,
    }
}
