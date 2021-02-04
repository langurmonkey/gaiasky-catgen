extern crate argparse;

use std::{
    collections::{HashMap, HashSet},
    f64::MAX,
};

use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};

use data::{Config, Particle};
use load::ColId;
use std::time::Instant;

mod color;
mod constants;
mod coord;
mod data;
mod load;
mod lod;
mod math;
mod parse;
mod util;
mod xmatch;

// Maximum number of files to load per catalog
const MAX_FILES: u64 = 3;
// Maximum number of records (stars) to load per file
const MAX_RECORDS: u64 = 50000;

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
        let mut start;
        // Complete inputs
        args.input.push_str("/*.gz");
        args.hip.push_str("/*.csv");

        // Load Hip-Gaia cross-match file
        // All stars with Hip counter-part are added to the
        // must_load list, which is later passed into the loader
        let mut must_load = HashSet::new();
        let mut xmatch_map = HashMap::new();
        if args.hip.len() > 0 && args.xmatch.len() > 0 {
            xmatch::load_xmatch(&args.xmatch, &mut xmatch_map);
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
            args.plx_zeropoint,
            args.ruwe_cap,
            args.distpc_cap,
            args.plx_err_faint,
            args.plx_err_bright,
            1.0,
            args.mag_corrections,
            false,
            Some(must_load),
            &args.additional,
            &args.columns,
        );
        // Actually load the catalog
        start = Instant::now();
        let list_gaia = loader_gaia
            .load_dir(&args.input)
            .expect("Error loading Gaia data");
        println!(
            "{} particles loaded form Gaia in {:?}",
            list_gaia.len(),
            start.elapsed(),
        );

        //
        // HIP - For hipparcos we only support the columns in that order
        //
        let loader_hip = load::Loader::new(
            1,
            u64::MAX,
            0.0,
            args.ruwe_cap,
            -1.0,
            1000.0,
            1000.0,
            1000.0,
            args.mag_corrections,
            true,
            None,
            "",
            "hip,names,ra,dec,plx,plx_err,pmra,pmdec,gmag,col_idx",
        );
        // Actually load hipparcos
        start = Instant::now();
        let mut list_hip = Vec::new();
        if args.hip.len() > 0 {
            list_hip = loader_hip
                .load_dir(&args.hip)
                .expect("Error loading HIP data");
            println!(
                "{} particles loaded form HIP in {:?}",
                list_hip.len(),
                start.elapsed()
            );
        }

        //
        // Merge Gaia and Hipparcos
        //
        let mut hip_map: HashMap<i32, &data::Particle> = HashMap::new();
        let mut main_list = Vec::new();
        let mut hip_added = HashSet::new();
        for hip_star in &list_hip {
            hip_map.insert(hip_star.hip, hip_star);
        }
        println!("{} stars added to hip_map", hip_map.len());
        let mut no_hit = 0;
        let mut hit = 0;
        let mut gaia_wins = 0;
        let mut hip_wins = 0;
        for mut gaia_star in list_gaia {
            if !xmatch_map.contains_key(&gaia_star.id) {
                // No hit, add directly to main list
                main_list.push(gaia_star);
                no_hit += 1;
            } else {
                // Hit, merge
                let hip_id = xmatch_map.get(&gaia_star.id).unwrap();
                if hip_map.contains_key(hip_id) {
                    hip_added.insert(hip_id);
                    let hip_star = hip_map.get(&hip_id).unwrap();
                    let gaia_plx_e = gaia_star.get_extra(load::ColId::plx_err);
                    let hip_plx_e = hip_star.get_extra(load::ColId::plx_err);

                    if gaia_plx_e <= hip_plx_e {
                        //println!("Gaia wins: {} <= {}", gaia_plx_e, hip_plx_e);
                        main_list.push(gaia_star);
                        gaia_wins += 1;
                    } else {
                        //println!("Hip wins: {} <= {}", hip_plx_e, gaia_plx_e);
                        main_list.push(hip_star.copy());
                        hip_wins += 1;
                    }
                } else {
                }
                hit += 1;
            }
        }
        let mut rest = 0;
        // Add rest of hip
        for hip_star in &list_hip {
            if !hip_added.contains(&hip_star.hip) {
                main_list.push(hip_star.copy());
                rest += 1;
            }
        }
        println!(
            "{} hits ({} gaia wins, {} hip wins), {} no-hits",
            hit, gaia_wins, hip_wins, no_hit
        );

        println!("{} stars in the final list", main_list.len());

        //
        // Actually generate LOD octree
        //
        start = Instant::now();
        println!("Sorting list with {} objects", main_list.len());
        main_list.sort_by(|a, b| a.absmag.partial_cmp(&b.absmag).unwrap());
        println!("List sorted in {:?}", start.elapsed());

        let mut octree = lod::Octree::new(
            args.max_part,
            args.postprocess,
            args.child_count,
            args.parent_count,
        );
        octree.generate(&main_list);

        //
        // Write tree and particles
        //
        std::process::exit(0);
    } else {
        eprintln!("Input catalog not specified!");
        std::process::exit(1);
    }
}
