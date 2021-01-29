mod color;
mod constants;
mod coord;
mod data;
mod load;
mod util;

#[macro_use]
extern crate lazy_static;

static ARG_RUWE_CAP: f32 = 1.4;
static ARG_DISTPC_CAP: f64 = -1.0;
static ARG_PLLX_ERR_FAINT: f64 = 0.99;
static ARG_PLLX_ERR_BRIGHT: f64 = 0.99;
static ARG_MAG_CORRECTIONS: bool = true;

fn main() {
    let list_hip = load::load_dir("data/hip/*.csv").expect("Error loading HIP data");
    println!("{} particles loaded successfully form HIP", list_hip.len());
    // Load Gaia
    let list_gaia = load::load_dir("data/gaia/*.gz").expect("Error loading Gaia data");
    println!(
        "{} particles loaded successfully form Gaia",
        list_gaia.len()
    );

    let n = std::cmp::min(50, list_gaia.len());
    for i in 0..n {
        let star = list_gaia.get(i).expect("Error getting star from list");
        println!("{}: [{},{},{}]", i, star.x, star.y, star.z);
    }
}
