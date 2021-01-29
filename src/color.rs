/**
 * Packs the color components into a u32 with the format ABGR
 **/
pub fn col_to_rgba8888(r: f32, g: f32, b: f32, a: f32) -> u32 {
    (((255.0 * a) as u32) << 24)
        | (((255.0 * b) as u32) << 16)
        | (((255.0 * g) as u32) << 8)
        | ((255.0 * r) as u32)
}

/**
 * Unpacks the RGBA888 u32 into the color components
 **/
pub fn rgba8888_to_col(value: u32) -> (f32, f32, f32, f32) {
    (
        ((value & 0xff000000) >> 24) as f32 / 255.0,
        ((value & 0x00ff0000) >> 16) as f32 / 255.0,
        ((value & 0x0000ff00) >> 8) as f32 / 255.0,
        ((value & 0x000000ff) as f32 / 255.0),
    )
}
