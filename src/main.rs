extern crate argparse;

use std::{
    collections::{HashMap, HashSet},
    path,
};

use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};

use constants::NEGATIVE_DIST;
use data::Config;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::Config as LogConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::fs;
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
mod write;
mod xmatch;

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
        distpc_cap: 1.0e6,
        plx_err_faint: 10.0,
        plx_err_bright: 10.0,
        plx_zeropoint: 0.0,
        mag_corrections: true,
        postprocess: false,
        child_count: 100,
        parent_count: 1000,
        file_num_cap: -1,
        star_num_cap: -1,
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
        ap.refer(&mut args.plx_zeropoint).add_option(
            &["--plxzeropoint"],
            Store,
            "Parallax zero point",
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
        ap.refer(&mut args.file_num_cap).add_option(
            &["--filescap"],
            Store,
            "Maximum number of input files to be processed",
        );
        ap.refer(&mut args.star_num_cap).add_option(
            &["--starscap"],
            Store,
            "Maximum number of stars to be processed per file",
        );
        ap.parse_args_or_exit();
    }

    let input_path = path::Path::new(&args.input);

    // Init logging
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y%m%d %H:%M:%S)} - {l} - {m}\n",
        )))
        .build(format!("{}/log", args.output))
        .expect("Error creating file appender");
    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y%m%d %H:%M:%S)} - {l} - {m}\n",
        )))
        .build();

    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("console", Box::new(console)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("console")
                .build(LevelFilter::Info),
        )
        .expect("Error building logger config");

    log4rs::init_config(config).expect("Error initializing logger");

    // Log arguments
    log::info!("{:?}", args);

    // Make sure input exists
    assert!(input_path.exists(), "Input directory does not exist");
    assert!(input_path.is_dir(), "Input directory is not a directory");

    if args.input.len() > 0 {
        let start = Instant::now();
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
            args.file_num_cap,
            args.star_num_cap,
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
        let start_gaia = Instant::now();
        let list_gaia = loader_gaia
            .load_dir(&args.input)
            .expect("Error loading Gaia data");
        let time_gaia = start_gaia.elapsed();
        log::info!(
            "{} particles loaded form Gaia in {:?}",
            list_gaia.len(),
            time_gaia,
        );

        //
        // HIP - For hipparcos we only support the columns in that order
        //
        let loader_hip = load::Loader::new(
            1,
            5000000,
            0.0,
            args.ruwe_cap,
            1e9,
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
        let mut list_hip = Vec::new();
        if args.hip.len() > 0 {
            println!("Load hip: {}", &args.hip);
            let start_hip = Instant::now();
            list_hip = loader_hip
                .load_dir(&args.hip)
                .expect("Error loading HIP data");
            let time_hip = start_hip.elapsed();
            log::info!(
                "{} particles loaded form HIP in {:?}",
                list_hip.len(),
                time_hip
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
        log::info!("{} stars added to hip_map", hip_map.len());
        let mut no_hit = 0;
        let mut hit = 0;
        let mut gaia_wins = 0;
        let mut hip_wins = 0;

        for gaia_star in list_gaia {
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
                        //log::info!("Gaia wins: {} <= {}", gaia_plx_e, hip_plx_e);
                        gaia_wins += 1;

                        let mut size = gaia_star.size;
                        let mut pos_gaia = data::Vec3::new(gaia_star.x, gaia_star.y, gaia_star.z);
                        let negative_dist = f64::abs(pos_gaia.len() - NEGATIVE_DIST) < 1e-10;
                        if negative_dist {
                            // Negative distance in gaia star
                            // use gaia 2D position, HIP distance and name

                            // Fetch Gaia RA/DEC
                            let gaia_sph =
                                util::cartesian_to_spherical(pos_gaia.x, pos_gaia.y, pos_gaia.z);
                            let gaia_ra = gaia_sph.x;
                            let gaia_dec = gaia_sph.y;

                            // Fetch HIP distance
                            let pos_hip = data::Vec3::new(hip_star.x, hip_star.y, hip_star.z);
                            let hip_sph =
                                util::cartesian_to_spherical(pos_hip.x, pos_hip.y, pos_hip.z);
                            let hip_dist = hip_sph.z;

                            // Compute new cartesian position
                            pos_gaia.set_from(&util::spherical_to_cartesian(
                                gaia_ra, gaia_dec, hip_dist,
                            ));

                            size = hip_star.size;
                        }

                        // Merged star
                        let mut star = hip_star.copy();
                        star.id = gaia_star.id;
                        // Pos
                        star.x = pos_gaia.x;
                        star.y = pos_gaia.y;
                        star.z = pos_gaia.z;
                        // Vel vector
                        star.pmx = gaia_star.pmx;
                        star.pmy = gaia_star.pmy;
                        star.pmz = gaia_star.pmz;
                        // Pm
                        star.mualpha = gaia_star.mualpha;
                        star.mudelta = gaia_star.mudelta;
                        star.radvel = gaia_star.radvel;
                        // Mag
                        star.appmag = gaia_star.appmag;
                        star.absmag = gaia_star.absmag;
                        // Col
                        star.col = gaia_star.col;
                        // Size
                        star.size = size;

                        main_list.push(star);
                    } else {
                        //log::info!("Hip wins: {} <= {}", hip_plx_e, gaia_plx_e);
                        main_list.push(hip_star.copy());
                        hip_wins += 1;
                    }
                }
                hit += 1;
            }
        }

        // Add rest of hip
        for hip_star in &list_hip {
            if !hip_added.contains(&hip_star.hip) {
                main_list.push(hip_star.copy());
            }
        }
        log::info!(
            "{} hits ({} gaia wins, {} hip wins), {} no-hits",
            hit,
            gaia_wins,
            hip_wins,
            no_hit
        );

        // Drop hip lists
        std::mem::drop(list_hip);

        log::info!("{} stars in the final list", main_list.len());
        let time_load = start.elapsed();

        if main_list.is_empty() {
            log::info!("No stars were loaded, aborting.");
            std::process::exit(1);
        }

        //
        // Actually generate LOD octree
        //
        let start_gen = Instant::now();
        let len_before = main_list.len();
        main_list.retain(|s| {
            let dist_pc: f64 = (s.x * s.x + s.y * s.y + s.z * s.z).sqrt() * constants::U_TO_PC;
            dist_pc <= args.distpc_cap
        });
        log::info!(
            "Removed {} stars due to being too far (cap = {} pc)",
            main_list.len() - len_before,
            args.distpc_cap
        );

        log::info!("Sorting list by magnitude with {} objects", main_list.len());
        main_list.sort_by(|a, b| a.absmag.partial_cmp(&b.absmag).unwrap());
        log::info!("List sorted in {:?}", start_gen.elapsed());

        let time_gen = start_gen.elapsed();

        let octree = lod::Octree::from_params(
            args.max_part,
            args.postprocess,
            args.child_count,
            args.parent_count,
            args.distpc_cap,
        );
        let (num_octants, num_stars, depth) = octree.generate_octree(&main_list);
        log::info!(
            "Octree generated with {}={} octants and {} stars ({} skipped) in {:?}",
            num_octants,
            octree.nodes.borrow().len(),
            num_stars,
            main_list.len() - num_stars,
            start_gen.elapsed()
        );
        octree.print();

        //
        // Write tree and particles
        //

        let start_write = Instant::now();
        fs::create_dir_all(&args.output).expect(&format!("Error creating dir: {}", args.output));

        // Write
        let main_list_len = main_list.len() as f32;
        write::write_metadata(&octree, &args.output);
        write::write_particles(&octree, main_list, &args.output);
        let time_write = start_write.elapsed();

        // Star counts per magnitude
        log::info!("");
        log::info!("=========================");
        log::info!("STAR COUNTS PER MAGNITUDE");
        log::info!("=========================");
        for i in 0..21 {
            let count =
                loader_hip.counts_per_mag.borrow()[i] + loader_gaia.counts_per_mag.borrow()[i];
            log::info!(
                "Magnitude {}: {} stars ({:.3}%)",
                i,
                count,
                (count as f32 * 100.0) / main_list_len
            )
        }

        // Octree stats
        log::info!("");
        log::info!("============");
        log::info!("OCTREE STATS");
        log::info!("============");
        log::info!("Octants: {}", num_octants);
        log::info!("Particles: {}", num_stars);
        log::info!("Depth: {}", depth);

        // Final stats
        log::info!("");
        log::info!("================");
        log::info!("FINAL TIME STATS");
        log::info!("================");
        log::info!("Loading: {:?}", time_load);
        log::info!("Generation: {:?}", time_gen);
        log::info!("Writing: {:?}", time_write);
        log::info!("Total: {:?}", start.elapsed());
        std::process::exit(0);
    } else {
        log::error!("Input catalog not specified!");
        std::process::exit(1);
    }
}
