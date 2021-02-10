//
//NATURE
//
pub const MILLIARCSEC_TO_ARCSEC: f64 = 1.0 / 1000.0;
pub const Y_TO_S: f64 = 31557600.0;
pub const S_TO_Y: f64 = 1.0 / Y_TO_S;

pub const PC_TO_KM: f64 = 3.08567758149137e13;
pub const PC_TO_M: f64 = PC_TO_KM * 1000.0;

//
//INTERNAL UNITS
//
pub const M_TO_U: f64 = 1e-9;
#[allow(dead_code)]
pub const U_TO_M: f64 = 1.0 / M_TO_U;

pub const KM_TO_U: f64 = M_TO_U * 1000.0;
#[allow(dead_code)]
pub const U_TO_KM: f64 = 1.0 / KM_TO_U;

pub const PC_TO_U: f64 = PC_TO_KM * KM_TO_U;
pub const U_TO_PC: f64 = 1.0 / PC_TO_U;

// Special, negative distance marker
pub const NEGATIVE_DIST: f64 = 1.0 * M_TO_U;
