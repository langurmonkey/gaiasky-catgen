extern crate nalgebra as na;

use na::base::Matrix4;

const R: f64 = 32.93192_f64;
const Q: f64 = 27.12825_f64;
const P: f64 = 192.85948_f64;

lazy_static! {
    pub static ref GAL_TO_EQ: Matrix4<f64> = Matrix4::from_euler_angles(
        -R.to_radians(),
        (90.0 - Q).to_radians(),
        (90.0 + P).to_radians(),
    );
    pub static ref EQ_TO_GAL: Matrix4<f64> = Matrix4::from_euler_angles(
        -R.to_radians(),
        (90.0 - Q).to_radians(),
        (90.0 + P).to_radians(),
    )
    .try_inverse()
    .unwrap();
}
