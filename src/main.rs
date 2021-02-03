extern crate argparse;

use std::collections::{HashMap, HashSet};

use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};

use data::Config;
use load::ColId;

mod color;
mod constants;
mod coord;
mod data;
mod load;
mod math;
mod parse;
mod util;
mod xmatch;

// Maximum number of files to load per catalog
const MAX_FILES: u64 = 2;
// Maximum number of records (stars) to load per file
const MAX_RECORDS: u64 = 5000;

/**
 * The main function parses the arguments, loads the Gaia and
 * Hipparcos catalogs (if needed, plus additional columns and metadata),
 * generates the LOD structure and writes it all to disk.
 **/
fn main() {
    // Arguments
    let mut args: Config = Config {
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
        columns: "source_id,ra,dec,plx,ra_err,dec_err,plx_err,pmra,pmdec,radvel,gmag,bpmag,rpmag,ruwe,ref_epoch".to_string(),
    };
    // Parse CLI arguments
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
        ap.refer(&mut args.columns).add_option(
            &["--columns"],
            Store,
            "Comma-separated list of column names, in order, of the Gaia catalog",
        );
        ap.parse_args_or_exit();
    }

    if args.input.len() > 0 {
        // Complete inputs
        args.input.push_str("/*.gz");
        args.hip.push_str("/*.csv");

        // Load Hip-Gaia cross-match file
        // All stars with Hip counter-part are added to the
        // must_load list, which is later passed into the loader
        let mut must_load = HashSet::new();
        if args.hip.len() > 0 && args.xmatch.len() > 0 {
            let xmatch_map = xmatch::load_xmatch(&args.xmatch);
            if !xmatch_map.is_empty() {
                let keys = xmatch_map.keys();
                for key in keys {
                    must_load.insert(*key);
                }
            }
        }

        //
        // GAIA - Load Gaia DRx catalog, the columns come from CLI arguments
        //
        let loader_gaia = load::Loader::new(
            MAX_FILES,
            MAX_RECORDS,
            &args,
            Some(must_load),
            &args.additional,
            &args.columns,
        );
        // Actually load the catalog
        let list_gaia = loader_gaia
            .load_dir(&args.input)
            .expect("Error loading Gaia data");
        println!(
            "{} particles loaded successfully form Gaia",
            list_gaia.len()
        );

        //
        // HIP - For hipparcos we only support the columns in that order
        //
        let loader_hip = load::Loader::new(
            MAX_FILES,
            MAX_RECORDS,
            &args,
            None,
            "",
            "hip,names,ra,dec,plx,plx_err,pmra,pmdec,gmag,col_idx",
        );
        // Actually load hipparcos
        if args.hip.len() > 0 {
            let list_hip = loader_hip
                .load_dir(&args.hip)
                .expect("Error loading HIP data");
            println!("{} particles loaded successfully form HIP", list_hip.len());
        }

        //
        // Merge Gaia and Hipparcos
        //

        //
        // Actually generate LOD octree
        //
        //

        //
        // Write tree and particles
        //
        std::process::exit(0);
    } else {
        eprintln!("Input catalog not specified!");
        std::process::exit(1);
    }
}
