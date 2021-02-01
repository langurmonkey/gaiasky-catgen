mod color;
mod constants;
mod coord;
mod data;
mod load;
mod math;
mod util;

#[macro_use]
extern crate lazy_static;
extern crate argparse;

use argparse::{ArgumentParser, Store, StoreTrue};

static ARG_RUWE_CAP: f32 = 1.4;
static ARG_DISTPC_CAP: f64 = -1.0;
static ARG_PLLX_ERR_FAINT: f64 = 0.99;
static ARG_PLLX_ERR_BRIGHT: f64 = 0.99;
static ARG_MAG_CORRECTIONS: bool = true;

fn main() {
    // Arguments
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Generate LOD catalogs for Gaia Sky.");
        ap.add_option(
            &["-v", "--version"],
            argparse::Print(env!("CARGO_PKG_VERSION").to_string()),
            "Print version information",
        );
        ap.parse_args_or_exit();
    }

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
    //let rgba8888 = color::col_to_rgba8888(0.15, 0.398, 0.43, 0.8);
    //println!("{:b}", rgba8888);
    //let (r, g, b, a) = color::rgba8888_to_col(9385193);
    //println!("{},{},{},{}", r, g, b, a);
}
