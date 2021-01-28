use crate::data;
use data::Vec3;

/**
 * Converts spherical coordinates (lon[rad], lat[rad], r) to
 * Cartesian coordinates in the same units as the radius.
 */
pub fn spherical_to_cartesian(lon: f64, lat: f64, r: f64) -> Vec3<f64> {
    // out.x = radius * Math.cos(latitude) * Math.sin(longitude);
    // out.y = radius * Math.sin(latitude);
    // out.z = radius * Math.cos(latitude) * Math.cos(longitude);
    let coslat = lat.cos();
    Vec3 {
        x: r * coslat * lon.sin(),
        y: r * lat.sin(),
        z: r * coslat * lon.cos(),
    }
}
