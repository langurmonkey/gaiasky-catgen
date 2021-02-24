use std::{f64::consts, time::Duration};

use data::Vec3;

use crate::constants;
use crate::data;

/**
 * Converts spherical coordinates (lon[rad], lat[rad], r) to
 * Cartesian coordinates in the same units as the radius.
 */
pub fn spherical_to_cartesian(lon: f64, lat: f64, r: f64) -> Vec3 {
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

pub fn cartesian_to_spherical(x: f64, y: f64, z: f64) -> Vec3 {
    let x2 = x * x;
    let y2 = y * y;
    let z2 = z * z;

    let distance = (x2 + y2 + z2).sqrt();

    let mut alpha = x.atan2(z);
    if alpha < 0.0 {
        alpha += 2.0 * consts::PI;
    }

    let delta;
    if x2 + z2 == 0.0 {
        delta = if y > 0.0 {
            consts::PI / 2.0
        } else {
            -consts::PI / 2.0
        };
    } else {
        delta = (y / (x2 + z2).sqrt()).atan();
    }

    Vec3 {
        x: alpha,
        y: delta,
        z: distance,
    }
}

pub fn propermotion_to_cartesian(
    mualphastar: f64,
    mudelta: f64,
    radvel: f64,
    ra_rad: f64,
    dec_rad: f64,
    dist_pc: f64,
) -> Vec3 {
    let ma = mualphastar * constants::MILLIARCSEC_TO_ARCSEC;
    let md = mudelta * constants::MILLIARCSEC_TO_ARCSEC;

    let vta = ma * dist_pc * 4.74;
    let vtd = md * dist_pc * 4.74;

    let cos_alpha = ra_rad.cos();
    let sin_alpha = ra_rad.sin();
    let cos_delta = dec_rad.cos();
    let sin_delta = dec_rad.sin();

    /*
     * vx = (vR cos \delta cos \alpha) - (vTA sin \alpha) - (vTD sin \delta cos \alpha)
     * vy = (vR cos \delta sin \alpha) + (vTA cos \alpha) - (vTD sin \delta sin \alpha)
     * vz = vR sin \delta + vTD cos \delta
     */
    let vx = (radvel * cos_delta * cos_alpha) - (vta * sin_alpha) - (vtd * sin_delta * cos_alpha);
    let vy = (radvel * cos_delta * sin_alpha) + (vta * cos_alpha) - (vtd * sin_delta * sin_alpha);
    let vz = (radvel * sin_delta) + (vtd * cos_delta);

    let to_u_per_y = constants::KM_TO_U / constants::S_TO_Y;

    Vec3 {
        x: vx * to_u_per_y,
        y: vy * to_u_per_y,
        z: vz * to_u_per_y,
    }
}

pub fn seconds_to_time(secs: u64) -> (u64, u64, u64) {
    let hours = secs / 3600;
    let mins = (secs - (hours * 3600)) / 60;
    let s = secs - (hours * 3600 + mins * 60);
    (hours, mins, s)
}

pub fn nice_time(duration: Duration) -> String {
    let (h, m, s) = seconds_to_time(duration.as_secs());
    format!("{} h, {} m, {} s", h, m, s)
}
