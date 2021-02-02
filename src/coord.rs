extern crate nalgebra as na;

use na::Matrix4;
use na::Vector3;

const R: f64 = 32.93192_f64;
const Q: f64 = 27.12825_f64;
const P: f64 = 192.85948_f64;

pub struct Coord {
    pub gal_eq: Matrix4<f64>,
    pub eq_gal: Matrix4<f64>,
}

impl Coord {
    pub fn new() -> Self {
        let axis_y = na::Unit::new_unchecked(Vector3::new(0.0, 1.0, 0.0));
        let axis_z = na::Unit::new_unchecked(Vector3::new(0.0, 0.0, 1.0));

        // Galactic <-> Equatorial
        let gal_eq_mat = Matrix4::from_axis_angle(&axis_y, f64::to_radians(90.0 + P))
            * Matrix4::from_axis_angle(&axis_z, f64::to_radians(90.0 - Q))
            * Matrix4::from_axis_angle(&axis_y, f64::to_radians(-R));
        Coord {
            gal_eq: gal_eq_mat,
            eq_gal: gal_eq_mat.try_inverse().unwrap(),
        }
    }
}
