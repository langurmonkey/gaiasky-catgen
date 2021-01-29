/**
 * Packs the color components into a 32-bit integer with the format ABGR and then converts it to a float. Alpha is compressed
 * from 0-255 to use only even numbers between 0-254 to avoid using float bits in the NaN range (see
 * int_to_float_color(i32)). Converting a color to a float and back can be lossy for alpha.
 * @return the packed color as a 32-bit float
 **/
pub fn to_int_bits(r: f32, g: f32, b: f32, a: f32) -> u32 {
    (((255.0 * a) as u32) << 24)
        | (((255.0 * b) as u32) << 16)
        | (((255.0 * g) as u32) << 8)
        | ((255.0 * r) as u32)
}

pub fn rgba8888_to_col(value: u32) -> (f32, f32, f32, f32) {
    (
        ((value & 0xff000000) >> 24) as f32 / 255.0,
        ((value & 0x00ff0000) >> 16) as f32 / 255.0,
        ((value & 0x0000ff00) >> 8) as f32 / 255.0,
        ((value & 0x000000ff) as f32 / 255.0),
    )
}
