//
//INTERNAL UNITS
//
pub const PC_TO_U: f64 = 3.08567758149137e16;
pub const M_TO_U: f64 = 1e-9;
pub const U_TO_M: f64 = 1.0 / M_TO_U;
pub const KM_TO_U: f64 = M_TO_U * 1000.0;
pub const PC_TO_KM: f64 = 3.08567758149137e13;
pub const PC_TO_M: f64 = PC_TO_KM * 1000.0;

//
//NATURE
//
pub const MILLIARCSEC_TO_ARCSEC: f64 = 1.0 / 1000.0;
pub const Y_TO_S: f64 = 31557600.0;
pub const S_TO_Y: f64 = 1.0 / Y_TO_S;

// Special, negative distance marker
pub const NEGATIVE_DIST: f64 = 1.0 * M_TO_U;
