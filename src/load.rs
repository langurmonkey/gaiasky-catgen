extern crate flate2;
extern crate nalgebra as na;

use crate::color;
use crate::constants;
use crate::coord;
use crate::data;
use crate::parse;
use crate::util;
use data::Args;
use data::Particle;
use flate2::read::GzDecoder;
use glob::glob;
use io::Write;
use na::base::Vector3;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::BufRead;
use std::{f32, f64, fs::File};

pub struct Additional {
    indices: HashMap<String, usize>,
    values: data::LargeLongMap<Vec<f64>>,
}

impl Additional {
    /**
     * Loads a new batch of additional columns from the given file
     **/
    pub fn new(file: &&str) -> Additional {
        Additional {
            indices: HashMap::new(),
            values: data::LargeLongMap::new(50),
        }
    }

    pub fn has_col(&self, col: &String) -> bool {
        self.indices.contains_key(col)
    }

    pub fn get(&mut self, col: String, source_id: i64) -> Option<f64> {
        if self.has_col(&col) {
            let index: usize = *self.indices.get(&col).expect("Error: could not get index");
            Some(*self.values.get(source_id).unwrap().get(index).unwrap())
        } else {
            None
        }
    }
}

pub struct Loader<'a> {
    // Maximum number of files to load in a directory
    pub max_files: u64,
    // Maximum number of records to load per file
    pub max_records: u64,
    // Arguments
    pub args: &'a Args,
    // Must-load star ids
    pub must_load: Option<HashSet<i64>>,
    // Additional columns
    pub additional: Vec<Additional>,
}

impl<'a> Loader<'a> {
    pub fn load_dir(&self, dir: &str) -> Result<Vec<Particle>, &'a str> {
        let mut list: Vec<Particle> = Vec::new();
        let mut i = 0;
        for entry in glob(&(dir.to_owned())).expect("Error reading glob pattern") {
            if i >= self.max_files {
                return Ok(list);
            }
            match entry {
                Ok(path) => {
                    print!("Processing: {:?}", path.display());
                    io::stdout().flush().expect("Error flushing stdout");
                    self.load_file(path.to_str().expect("Error: path not valid"), &mut list);
                }
                Err(e) => println!("Error: {:?}", e),
            }
            i += 1;
        }
        Ok(list)
    }

    // Loads a single file, being it csv.gz or csv
    // The format is hardcoded for now, with csv.gz being in eDR3 format,
    // and csv being in hipparcos format.
    pub fn load_file(&self, file: &str, list: &mut Vec<Particle>) {
        let mut total: u64 = 0;
        let mut loaded: u64 = 0;
        let mut skipped: u64 = 0;
        if file.ends_with(".gz") || file.ends_with(".gzip") {
            // Read csv.gz using GzDecoder
            let f = File::open(file).expect("Error: file not found");
            let gz = GzDecoder::new(f);

            for line in io::BufReader::new(gz).lines() {
                // Skip header
                if total > 0 {
                    match self.parse_line_gz(line.expect("Error reading line")) {
                        Some(part) => {
                            list.push(part);
                            loaded += 1;
                        }
                        None => skipped += 1,
                    }
                }
                total += 1;
                if total - 1 >= self.max_records {
                    break;
                }
            }
            self.log_file(loaded, total, skipped);
        } else if file.ends_with(".csv") || file.ends_with(".txt") {
            // Read plain text file
            let f = File::open(file).expect("Error: file not found");

            for line in io::BufReader::new(f).lines() {
                // Skip header
                if total > 0 {
                    match self.parse_line_csv(line.expect("Error reading line")) {
                        Some(part) => {
                            list.push(part);
                            loaded += 1;
                        }
                        None => skipped += 1,
                    }
                }
                total += 1;
                if total - 1 >= self.max_records {
                    break;
                }
            }
            self.log_file(loaded, total, skipped);
        }
    }

    fn log_file(&self, loaded: u64, total: u64, skipped: u64) {
        println!(
            " - loaded {}/{} objects ({:.3}%, {} skipped)",
            loaded,
            total - 1,
            loaded as f32 / (total as f32 - 1.0),
            skipped
        );
    }

    // Parses a hipparcos csv file and returns a Particle
    // 0:hip,1:name,2:ra,3:dec,4:plx,5:e_plx,6:pmra,7:pmde,8:mag,9:b_v
    fn parse_line_csv(&self, line: String) -> Option<Particle> {
        let tokens: Vec<&str> = line.split(',').collect();

        self.create_particle(
            tokens.get(0),
            tokens.get(0),
            tokens.get(2),
            tokens.get(3),
            tokens.get(4),
            tokens.get(5),
            tokens.get(6),
            tokens.get(7),
            None,
            tokens.get(8),
            None,
            None,
            tokens.get(9),
            None,
        )
    }

    // Parses a line in eDR3 csv.gz format and returns a Particle
    // 0:sourceid,1:ra,2:dec,3:plx,4:e_ra,5:e_dec,6:e_plx,7:pmra,8:pmde,9:rv,10:gmag,11:bp,12:rp,13:ruwe,14:ref_epoch
    fn parse_line_gz(&self, line: String) -> Option<Particle> {
        let tokens: Vec<&str> = line.split(',').collect();

        self.create_particle(
            tokens.get(0),
            Some(&"-1"),
            tokens.get(1),
            tokens.get(2),
            tokens.get(3),
            tokens.get(6),
            tokens.get(7),
            tokens.get(8),
            tokens.get(9),
            tokens.get(10),
            tokens.get(11),
            tokens.get(12),
            None,
            tokens.get(13),
        )
    }

    fn must_load_particle(&self, id: i64) -> bool {
        match &self.must_load {
            Some(must_load) => must_load.contains(&id),
            None => false,
        }
    }

    fn create_particle(
        &self,
        ssource_id: Option<&&str>,
        ship_id: Option<&&str>,
        sra: Option<&&str>,
        sdec: Option<&&str>,
        splx: Option<&&str>,
        splx_e: Option<&&str>,
        spmra: Option<&&str>,
        spmdec: Option<&&str>,
        srv: Option<&&str>,
        sappmag: Option<&&str>,
        sbp: Option<&&str>,
        srp: Option<&&str>,
        sbv: Option<&&str>,
        sruwe: Option<&&str>,
    ) -> Option<Particle> {
        // First, check if we accept it given the current constraints
        let plx: f64 = parse::parse_f64(splx);
        let plx_e: f64 = parse::parse_f64(splx_e);
        let mut appmag: f64 = parse::parse_f64(sappmag);

        // Source ID
        let source_id: i64 = parse::parse_i64(ssource_id);

        let must_load = self.must_load_particle(source_id);

        // Parallax test
        if !must_load && !self.accept_parallax(appmag, plx, plx_e) {
            return None;
        }

        let ruwe_val: f32 = parse::parse_f32(sruwe);
        // RUWE test
        if !must_load && !self.accept_ruwe(ruwe_val) {
            return None;
        }

        // Distance
        let dist_pc: f64 = 1000.0 / plx;

        // Distance test
        if !must_load && !self.accept_distance(dist_pc) {
            return None;
        }
        let dist: f64 = dist_pc * constants::PC_TO_U;

        // Name
        let name_vec = None;

        // RA and DEC
        let ra: f64 = parse::parse_f64(sra);
        let dec: f64 = parse::parse_f64(sdec);

        let pos = util::spherical_to_cartesian(
            ra.to_radians(),
            dec.to_radians(),
            dist.max(constants::NEGATIVE_DIST),
        );

        // Proper motions
        let mualphastar: f64 = parse::parse_f64(spmra);
        let mudelta: f64 = parse::parse_f64(spmdec);
        let mut radvel: f64 = parse::parse_f64(srv);
        if radvel.is_nan() {
            radvel = 0.0;
        }

        let pm = util::propermotion_to_cartesian(
            mualphastar,
            mudelta,
            radvel,
            ra.to_radians(),
            dec.to_radians(),
            dist_pc,
        );

        // Apparent magnitudes
        let mut ag: f64 = 0.0;
        let pos_gal: Vector3<f64> = Vector3::new(pos.x, pos.y, pos.z);
        coord::EQ_TO_GAL.transform_vector(&pos_gal);
        let pos_gal_sph = util::cartesian_to_spherical(pos_gal.x, pos_gal.y, pos_gal.z);
        let b = pos_gal_sph.y;
        let magcorraux = dist_pc.min(150.0 / b.sin().abs());

        if self.args.mag_corrections {
            ag = magcorraux * 5.9e-4;
            ag = ag.min(3.2);
        }
        if ag.is_finite() {
            appmag -= ag;
        }

        // Absolute magnitude
        let absmag = appmag - 5.0 * (if dist_pc <= 0.0 { 10.0 } else { dist_pc }).log10() + 5.0;

        // Size
        let pseudo_l = 10.0_f64.powf(-0.4 * absmag);
        let size_fac = constants::PC_TO_M * constants::M_TO_U * 0.15;
        let size: f32 = 1.0e10_f64.powf(pseudo_l.powf(0.45) * size_fac) as f32;

        // Color
        let ebr: f64 = 0.0;

        let col_idx: f64;
        let teff: f64;
        if sbp.is_some() && srp.is_some() {
            let bp: f64 = parse::parse_f64(sbp);
            let rp: f64 = parse::parse_f64(srp);
            col_idx = bp - rp - ebr;

            teff = color::xp_to_teff(col_idx);
        } else if sbv.is_some() {
            col_idx = parse::parse_f64(sbv);
            teff = color::bv_to_teff_ballesteros(col_idx);
        } else {
            col_idx = 0.656;
            teff = color::bv_to_teff_ballesteros(col_idx);
        }
        let (col_r, col_g, col_b) = color::teff_to_rgb(teff);
        let col: u32 = color::col_to_rgba8888(col_r as f32, col_g as f32, col_b as f32, 1.0);

        // HIP
        let hip_id: i32 = parse::parse_i32(ship_id);

        Some(Particle {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            pmx: pm.x as f32,
            pmy: pm.y as f32,
            pmz: pm.z as f32,
            mualpha: mualphastar as f32,
            mudelta: mudelta as f32,
            radvel: radvel as f32,
            appmag: appmag as f32,
            absmag: absmag as f32,
            col: col,
            size: size,
            hip: hip_id,
            id: source_id,
            names: name_vec,
        })
    }

    fn accept_parallax(&self, appmag: f64, plx: f64, plx_e: f64) -> bool {
        if !appmag.is_finite() {
            return false;
        } else if appmag < 13.1 {
            return plx >= 0.0 && plx_e < plx * self.args.plx_err_bright && plx_e <= 1.0;
        } else {
            return plx >= 0.0 && plx_e < plx * self.args.plx_err_faint && plx_e <= 1.0;
        }
    }

    fn accept_distance(&self, dist_pc: f64) -> bool {
        self.args.distpc_cap <= 0.0 || dist_pc <= self.args.distpc_cap
    }

    fn accept_ruwe(&self, ruwe: f32) -> bool {
        ruwe.is_nan() || self.args.ruwe_cap.is_nan() || ruwe < self.args.ruwe_cap
    }
}
