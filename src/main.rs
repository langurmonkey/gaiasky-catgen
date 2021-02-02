mod color;
mod constants;
mod coord;
mod data;
mod load;
mod math;
mod parse;
mod util;
mod xmatch;

#[macro_use]
extern crate lazy_static;
extern crate argparse;

use data::Args;

use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};
use std::collections::{HashMap, HashSet};

fn main() {
    // Arguments
    let mut args: Args = Args {
        input: "".to_string(),
        output: "".to_string(),
        max_part: 100000,
        ruwe_cap: f32::NAN,
        distpc_cap: -1.0,
        plx_err_faint: 10.0,
        plx_err_bright: 10.0,
        plx_zeropoint: 0.0,
        mag_corrections: true,
        postprocess: false,
        child_count: 100,
        parent_count: 1000,
        hip: "".to_string(),
        additional: "".to_string(),
        xmatch: "".to_string(),
    };
    // Parse
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Generate LOD catalogs for Gaia Sky.");
        ap.add_option(
            &["-v", "--version"],
            argparse::Print(env!("CARGO_PKG_VERSION").to_string()),
            "Print version information",
        );
        ap.refer(&mut args.input).add_option(
            &["-i", "--input"],
            Store,
            "Location of the input catalog",
        );
        ap.refer(&mut args.output).add_option(
            &["-o", "--output"],
            Store,
            "Output folder. Defaults to system temp",
        );
        ap.refer(&mut args.max_part).add_option(
            &["--maxpart"],
            Store,
            "Maximum number of objects in an octant",
        );
        ap.refer(&mut args.plx_err_faint).add_option(
            &["--plxerrfaint"],
            Store,
            "Parallax error factor for faint stars (gmag>=13.1), where filter [plx_err/plx < plxerrfaint] is enforced",
        );
        ap.refer(&mut args.plx_err_bright).add_option(
            &["--plxerrbright"],
            Store,
            "Parallax error factor for bright stars (gmag<13.1), where filter [plx_err/plx < plxerrbright] is enforced",
        );
        ap.refer(&mut args.mag_corrections).add_option(
            &["-c", "--skipmagcorrections"],
            StoreFalse,
            "Skip magnitude and color corrections for extinction and reddening",
        );
        ap.refer(&mut args.postprocess).add_option(
            &["-p", "--postprocess"],
            StoreTrue,
            "Post-process tree so that low-count nodes are merged with their parents. See --childcount and --parentcount for more info",
        );
        ap.refer(&mut args.child_count).add_option(
            &["--childcount"],
            Store,
            "If --postprocess is on, children nodes with less than --childcount objects and whose parent has less than --parentcount objects will be merged with their parent. Defaults to 100",
        );
        ap.refer(&mut args.parent_count).add_option(
            &["--parentcount"],
            Store,
            "If --postprocess is on, children nodes with less than --childcount objects and whose parent has less than --parentcount objects will be merged with their parent. Defaults to 1000",
        );
        ap.refer(&mut args.hip).add_option(
            &["--hip"],
            Store,
            "Absolute or relative location of the Hipparcos catalog (only csv supported)",
        );
        ap.refer(&mut args.distpc_cap).add_option(
            &["--distcap"],
            Store,
            "Maximum distance in parsecs. Stars beyond this limit are ignored",
        );
        ap.refer(&mut args.additional).add_option(
            &["--additional"],
            Store,
            "Comma-separated list of files or folders with optionally gzipped csv files containing additional columns (matched by id) of the main catalog. The first column must contain the Gaia source_id",
        );
        ap.refer(&mut args.xmatch).add_option(
            &["--xmatchfile"],
            Store,
            "Crossmatch file between Gaia and Hipparcos, containing two columns: source_id and hip",
        );
        ap.refer(&mut args.ruwe_cap).add_option(
            &["--ruwe"],
            Store,
            "RUWE threshold value. Filters out all stars with RUWE greater than this value. If present, --plxerrfaint and --plxerrbright are ignored.",
        );
        ap.parse_args_or_exit();
    }

    if args.input.len() > 0 {
        // Complete inputs
        args.input.push_str("/*.gz");
        args.hip.push_str("/*.csv");

        let mut must_load = HashSet::new();
        // xmatch helps us construct the must_load ids
        if args.hip.len() > 0 && args.xmatch.len() > 0 {
            let xmatch_map = xmatch::load_xmatch(&args.xmatch);
            if !xmatch_map.is_empty() {
                let keys = xmatch_map.keys();
                for key in keys {
                    must_load.insert(*key);
                }
            }
        }
        // Load additional, if any
        let mut additional = Vec::new();
        if !args.additional.is_empty() {
            let tokens: Vec<&str> = args.additional.split(',').collect();
            for token in tokens {
                let add = load::Additional::new(&token).expect("Error loading additional");
                println!("Loaded {} columns and {} records from {}", add.n_cols(), add.size(), token);
                additional.push(add);
            }
        }

        //
        // GAIA
        //
        let mut indices_gaia = HashMap::new();
        indices_gaia.insert(load::ColId::source_id, 0);
        indices_gaia.insert(load::ColId::ra, 1);
        indices_gaia.insert(load::ColId::dec, 2);
        indices_gaia.insert(load::ColId::plx, 3);
        indices_gaia.insert(load::ColId::ra_err, 4);
        indices_gaia.insert(load::ColId::dec_err, 5);
        indices_gaia.insert(load::ColId::plx_err, 6);
        indices_gaia.insert(load::ColId::pmra, 7);
        indices_gaia.insert(load::ColId::pmdec, 8);
        indices_gaia.insert(load::ColId::radvel, 9);
        indices_gaia.insert(load::ColId::gmag, 10);
        indices_gaia.insert(load::ColId::bpmag, 11);
        indices_gaia.insert(load::ColId::rpmag, 12);
        indices_gaia.insert(load::ColId::ruwe, 13);
        indices_gaia.insert(load::ColId::ref_epoch, 14);
        let loader_gaia = load::Loader {
            max_files: 1,
            max_records: 5,
            args: &args,
            must_load: Some(must_load),
            additional: additional,
            indices: indices_gaia,
        };
        // Load Gaia
        let list_gaia = loader_gaia
            .load_dir(&args.input)
            .expect("Error loading Gaia data");
        println!(
            "{} particles loaded successfully form Gaia",
            list_gaia.len()
        );

        //
        // HIPPARCOS
        //
        let mut indices_hip = HashMap::new();
        indices_hip.insert(load::ColId::hip, 0);
        indices_hip.insert(load::ColId::source_id, 0);
        indices_hip.insert(load::ColId::names, 1);
        indices_hip.insert(load::ColId::ra, 2);
        indices_hip.insert(load::ColId::dec, 3);
        indices_hip.insert(load::ColId::plx, 4);
        indices_hip.insert(load::ColId::plx_err, 5);
        indices_hip.insert(load::ColId::pmra, 6);
        indices_hip.insert(load::ColId::pmdec, 7);
        indices_hip.insert(load::ColId::gmag, 8);
        indices_hip.insert(load::ColId::col_idx, 9);
        let loader_hip = load::Loader {
            max_files: 1,
            max_records: 5,
            args: &args,
            must_load: None,
            additional: Vec::new(),
            indices: indices_hip,
        };
        // Load HIP
        if args.hip.len() > 0 {
            let list_hip = loader_hip
                .load_dir(&args.hip)
                .expect("Error loading HIP data");
            println!("{} particles loaded successfully form HIP", list_hip.len());
        }

        let n = std::cmp::min(50, list_gaia.len());
        for i in 0..n {
            let star = list_gaia.get(i).expect("Error getting star from list");
            println!("{}: [{},{},{}]", i, star.x, star.y, star.z);
        }
        std::process::exit(0);
    } else {
        eprintln!("Input catalog not specified!");
        std::process::exit(1);
    }
}
