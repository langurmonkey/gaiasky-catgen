use crate::math;
use std::mem;

const A: f64 = 0.92;
const B: f64 = 1.7;
const C: f64 = 0.62;
const T0: f64 = 4600.0;

// Ballesteros 2012 (EPL 97, 34008) conversion between T_eff and B-V.
pub fn bv_to_teff_ballesteros(bv: f64) -> f64 {
    T0 * (1.0 / (A * bv + B) + 1.0 / (A * bv + C))
}

// XP to T_eff conversion by Jordi et al.
pub fn xp_to_teff(xp: f64) -> f64 {
    if xp <= 1.5 {
        f64::powf(
            10.0,
            3.999 - 0.654 * xp + 0.709 * f64::powf(xp, 2.0) - 0.316 * f64::powf(xp, 3.0),
        )
    } else {
        math::lint(xp, 1.5, 15.0, 3521.6, 3000.0)
    }
}

// Converts effective temperature in Kelvin (1000-40000) to RGB.
//
// See https://www.tannerhelland.com/4435/convert-temperature-rgb-algorithm-code/ -- T_eff to RGB.
pub fn teff_to_rgb(teff: f64) -> (f32, f32, f32) {
    let mut r: f64;
    let mut g: f64;
    let mut b: f64;

    let temp: f64 = teff / 100.0;

    // Red
    if temp <= 66.0 {
        r = 255.0;
    } else {
        let x: f64 = temp - 55.0;
        r = 351.97690566805693 + 0.114206453784165 * x - 40.25366309332127 * f64::ln(x);
        r = r.clamp(0.0, 255.0);
    }

    // Green
    if temp <= 66.0 {
        let x: f64 = temp - 2.0;
        g = -155.25485562709179 - 0.44596950469579133 * x + 104.49216199393888 * f64::ln(x);
        g = g.clamp(0.0, 255.0);
    } else {
        let x: f64 = temp - 50.0;
        g = 325.4494125711974 + 0.07943456536662342 * x - 28.0852963507957 * f64::ln(x);
        g = g.clamp(0.0, 255.0);
    }

    // Blue
    if temp >= 66.0 {
        b = 255.0;
    } else {
        if temp <= 19.0 {
            b = 0.0;
        } else {
            let x = temp - 10.0;
            b = -254.76935184120902 + 0.8274096064007395 * x + 115.67994401066147 * f64::ln(x);
            b = b.clamp(0.0, 255.0);
        }
    }

    ((r / 255.0) as f32, (g / 255.0) as f32, (b / 255.0) as f32)
}

/**
 * Packs the color components into an i32 with the format ABGR8888
 **/
pub fn col_to_f32(r: f32, g: f32, b: f32, a: f32) -> f32 {
    let col = (((255.0 * a) as u32) << 24)
        | (((255.0 * b) as u32) << 16)
        | (((255.0 * g) as u32) << 8)
        | ((255.0 * r) as u32);
    i32_to_f32_color(col)
}

pub fn i32_to_f32_color(value: u32) -> f32 {
    i32_bits_to_f32(value & 0xfeffffff)
}
pub fn i32_bits_to_f32(value: u32) -> f32 {
    let bytes = value.to_ne_bytes();
    unsafe { mem::transmute::<[u8; 4], f32>(bytes) }
}

/**
 * Packs the color components into an i32 with the format RGBA8888
 **/
#[allow(dead_code)]
pub fn col_to_rgba8888(r: f32, g: f32, b: f32, a: f32) -> i32 {
    (((255.0 * r) as i32) << 24)
        | (((255.0 * g) as i32) << 16)
        | (((255.0 * b) as i32) << 8)
        | ((255.0 * a) as i32)
}

/**
 * Unpacks the RGBA888 i32 into the color components
 **/
#[allow(dead_code)]
pub fn rgba8888_to_col(value: i32) -> (f32, f32, f32, f32) {
    let val: u32 = value as u32;
    (
        ((val & 0xff000000) >> 24) as f32 / 255.0,
        ((val & 0x00ff0000) >> 16) as f32 / 255.0,
        ((val & 0x0000ff00) >> 8) as f32 / 255.0,
        ((val & 0x000000ff) as f32 / 255.0),
    )
}
