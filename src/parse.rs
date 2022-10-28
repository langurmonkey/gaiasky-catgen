extern crate fast_float;

pub fn parse_i32(val_str: Option<&&str>) -> i32 {
    match val_str {
        Some(val) => val.parse::<i32>().unwrap_or(0),
        None => 0,
    }
}

pub fn parse_i64(val_str: Option<&&str>) -> i64 {
    match val_str {
        Some(val) => val.parse::<i64>().unwrap_or(0),
        None => 0,
    }
}

pub fn parse_f32(val_str: Option<&&str>) -> f32 {
    match val_str {
        Some(val) => fast_float::parse(val).unwrap_or(f32::NAN),
        None => f32::NAN,
    }
}

pub fn parse_f64(val_str: Option<&&str>) -> f64 {
    match val_str {
        Some(val) => fast_float::parse(val).unwrap_or(f64::NAN),
        None => f64::NAN,
    }
}

/// Checks whether the given optional string has an actual value inside.
/// In essence, returns true if the option is Some and its length is not 0.
pub fn is_empty(val_str: Option<&&str>) -> bool {
    match val_str {
        Some(val) => val.len() == 0,
        None => true,
    }
}
