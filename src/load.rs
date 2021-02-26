extern crate flate2;
extern crate memmap;
extern crate nalgebra as na;
extern crate regex;

use crate::color;
use crate::constants;
use crate::coord;
use crate::data;
use crate::parse;
use crate::util;

use memmap::Mmap;
use regex::Regex;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::io::{BufRead, Read};
use std::path::Path;
use std::{f32, f64, fs::File};

use flate2::read::GzDecoder;
use glob::glob;
use na::base::Vector3;

use data::Particle;

#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ColId {
    source_id,
    hip,
    names,
    ra,
    dec,
    plx,
    ra_err,
    dec_err,
    plx_err,
    pmra,
    pmdec,
    radvel,
    pmra_err,
    pmdec_err,
    radvel_err,
    gmag,
    bpmag,
    rpmag,
    bp_rp,
    col_idx,
    ref_epoch,
    teff,
    radius,
    ag,
    ebp_min_rp,
    ruwe,
    geodist,
}

impl ColId {
    pub fn to_str(&self) -> &str {
        match self {
            ColId::source_id => "source_id",
            ColId::hip => "hip",
            ColId::names => "names",
            ColId::ra => "ra",
            ColId::dec => "dec",
            ColId::plx => "plx",
            ColId::ra_err => "ra_err",
            ColId::dec_err => "dec_err",
            ColId::plx_err => "plx_err",
            ColId::pmra => "pmra",
            ColId::pmdec => "pmdec",
            ColId::radvel => "radvel",
            ColId::radvel_err => "radvel_err",
            ColId::gmag => "gmag",
            ColId::bpmag => "bpmag",
            ColId::rpmag => "rpmag",
            ColId::bp_rp => "bp_rp",
            ColId::col_idx => "col_idx",
            ColId::ref_epoch => "ref_epoch",
            ColId::ruwe => "ruwe",
            ColId::teff => "teff",
            ColId::ag => "ag",
            ColId::ebp_min_rp => "ebp_min_rp",
            ColId::geodist => "geodist",
            _ => "*none*",
        }
    }
    pub fn from_str(input: &str) -> Option<ColId> {
        match input {
            "source_id" => Some(ColId::source_id),
            "sourceid" => Some(ColId::source_id),
            "hip" => Some(ColId::hip),
            "names" => Some(ColId::names),
            "name" => Some(ColId::names),
            "ra" => Some(ColId::ra),
            "dec" => Some(ColId::dec),
            "de" => Some(ColId::dec),
            "plx" => Some(ColId::plx),
            "pllx" => Some(ColId::plx),
            "parallax" => Some(ColId::plx),
            "ra_error" => Some(ColId::ra_err),
            "ra_err" => Some(ColId::ra_err),
            "ra_e" => Some(ColId::ra_err),
            "dec_error" => Some(ColId::dec_err),
            "dec_err" => Some(ColId::dec_err),
            "dec_e" => Some(ColId::dec_err),
            "de_error" => Some(ColId::dec_err),
            "de_err" => Some(ColId::dec_err),
            "de_e" => Some(ColId::dec_err),
            "plx_error" => Some(ColId::plx_err),
            "plx_err" => Some(ColId::plx_err),
            "plx_e" => Some(ColId::plx_err),
            "pllx_error" => Some(ColId::plx_err),
            "pllx_err" => Some(ColId::plx_err),
            "pllx_e" => Some(ColId::plx_err),
            "pmra" => Some(ColId::pmra),
            "pmdec" => Some(ColId::pmdec),
            "pmde" => Some(ColId::pmdec),
            "radvel" => Some(ColId::radvel),
            "rv" => Some(ColId::radvel),
            "radvel_err" => Some(ColId::radvel_err),
            "radvel_e" => Some(ColId::radvel_err),
            "rv_err" => Some(ColId::radvel_err),
            "rv_e" => Some(ColId::radvel_err),
            "gmag" => Some(ColId::gmag),
            "appmag" => Some(ColId::gmag),
            "bpmag" => Some(ColId::bpmag),
            "bp" => Some(ColId::bpmag),
            "rpmag" => Some(ColId::rpmag),
            "rp" => Some(ColId::rpmag),
            "bp_rp" => Some(ColId::bp_rp),
            "bp-rp" => Some(ColId::bp_rp),
            "col_idx" => Some(ColId::col_idx),
            "b_v" => Some(ColId::col_idx),
            "b-v" => Some(ColId::col_idx),
            "ref_epoch" => Some(ColId::ref_epoch),
            "teff" => Some(ColId::teff),
            "t_eff" => Some(ColId::teff),
            "T_eff" => Some(ColId::teff),
            "ruwe" => Some(ColId::ruwe),
            "ag" => Some(ColId::ag),
            "ebp_min_rp" => Some(ColId::ebp_min_rp),
            "geodist" => Some(ColId::geodist),
            _ => None,
        }
    }
}

/**
 * This holds additional columns for this loader.
 * The columns are in a large map whose keys are
 * source IDs. The column data is a vector of f64.
 * The index in the vector corresponding to each
 * column can be found in the map indices.
 **/
pub struct Additional {
    pub indices: HashMap<String, usize>,
    pub values: BTreeMap<i64, Vec<f64>>,
}

impl Additional {
    /**
     * Loads a new batch of additional columns from the given file
     **/
    pub fn new(file: &&str) -> Option<Self> {
        log::info!("Loading additional columns from {}", file);

        let path = Path::new(file);
        if path.exists() {
            if path.is_file() && file.ends_with(".gz") {
                return Additional::load_file(file);
            } else {
                log::error!("Path is not a gzipped file: {}", file);
                return None;
            }
        } else {
            log::error!("Path does not exist: {}", file);
            return None;
        }
    }

    fn load_file(file: &str) -> Option<Self> {
        let mut total: u64 = 0;
        // Read csv.gz using GzDecoder
        let f = File::open(file).expect("Error: file not found");
        let mmap = unsafe { Mmap::map(&f).expect(&format!("Error mapping file {}", file)) };
        let gz = GzDecoder::new(&mmap[..]);

        let mut indices = HashMap::new();
        let mut values = BTreeMap::new();

        // Separator, multiple spaces or a comma
        let sep = Regex::new(r"\s+|,").unwrap();

        for line in io::BufReader::new(gz).lines() {
            if total == 0 {
                // Header
                let line_str = line.expect("Error reading line");
                let tokens: Vec<&str> = sep.split(&line_str).collect();
                let mut i = 0;
                for token in tokens {
                    if i == 0 && !token.eq("source_id") && !token.eq("sourceid") {
                        log::error!(
                            "Error: first column '{}' of additional must be '{}'",
                            token,
                            ColId::source_id.to_str()
                        );
                        return None;
                    }
                    if i > 0 && !indices.contains_key(token) {
                        indices.insert(token.to_string(), i - 1);
                    }
                    i += 1;
                }
            } else {
                // Data
                let line_str = line.expect("Error reading line");
                let tokens: Vec<&str> = sep.split(&line_str).collect();
                let source_id: i64 = parse::parse_i64(tokens.get(0));
                let ncols = tokens.len();

                // Vector with values
                let mut vals: Vec<f64> = Vec::new();

                for j in 1..ncols {
                    let token = tokens.get(j);
                    if token.is_none() {
                        vals.push(f64::NAN);
                    } else {
                        if token.unwrap().trim().is_empty() {
                            vals.push(f64::NAN);
                        } else {
                            // Actual value
                            let val = parse::parse_f64(token);
                            vals.push(val);
                        }
                    }
                }
                values.insert(source_id, vals);
            }
            total += 1;
        }

        Some(Additional { indices, values })
    }

    pub fn n_cols(&self) -> usize {
        self.indices.len()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn has_col(&self, col_id: ColId) -> bool {
        self.indices.contains_key(col_id.to_str())
    }

    pub fn get(&self, col_id: ColId, source_id: i64) -> Option<f64> {
        if self.has_col(col_id) {
            let index: usize = *self
                .indices
                .get(col_id.to_str())
                .expect("Error: could not get index");
            let sid = self.values.get(&source_id);
            if sid.is_some() {
                let v = sid.unwrap().get(index);
                if v.is_some() {
                    Some(v.unwrap().clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct Loader {
    // Maximum number of files to load in a directory
    pub max_files: i32,
    // Maximum number of records to load per file
    pub max_records: i32,
    // Zero point for parallaxes
    pub plx_zeropoint: f64,
    // RUWE cap value
    pub ruwe_cap: f32,
    // Distance cap in parsecs
    pub distpc_cap: f64,
    // plx_error criteria for faint stars
    pub plx_err_faint: f64,
    // plx_error criteria for bright stars
    pub plx_err_bright: f64,
    // Cap on the parallax error (stars with larger plx_err are discarded)
    pub plx_err_cap: f64,
    // Whether to apply mag corrections
    pub mag_corrections: bool,
    // If set to true, negative parallaxes will be transformed to the default 0.04 arcsec value
    pub allow_negative_plx: bool,
    // Must-load star ids
    pub must_load: Option<HashSet<i64>>,
    // Additional columns
    pub additional: Vec<Additional>,
    // Indices
    pub indices: HashMap<ColId, usize>,
    // Coordinate conversion
    pub coord: coord::Coord,
    // Star counts per magnitude
    pub counts_per_mag: RefCell<[u32; 22]>,
}

impl Loader {
    pub fn new(
        max_files: i32,
        max_records: i32,
        plx_zeropoint: f64,
        ruwe_cap: f32,
        distpc_cap: f64,
        plx_err_faint: f64,
        plx_err_bright: f64,
        plx_err_cap: f64,
        mag_corrections: bool,
        allow_negative_plx: bool,
        must_load: Option<HashSet<i64>>,
        additional_str: &str,
        indices_str: &str,
    ) -> Self {
        // Additional
        let mut additional = Vec::new();
        if !additional_str.is_empty() {
            let tokens: Vec<&str> = additional_str.split(',').collect();
            for token in tokens {
                let add = Additional::new(&token).expect("Error loading additional");
                log::info!(
                    "Loaded {} columns and {} records from {}",
                    add.n_cols(),
                    add.len(),
                    token
                );
                additional.push(add);
            }
        }

        // Indices
        let mut indices = HashMap::new();
        let cols: Vec<&str> = indices_str.split(',').collect();
        for i in 0..cols.len() {
            indices.insert(ColId::from_str(cols.get(i).unwrap()).unwrap(), i);
        }

        Loader {
            max_files,
            max_records,
            plx_zeropoint,
            ruwe_cap,
            distpc_cap,
            plx_err_faint,
            plx_err_bright,
            plx_err_cap,
            mag_corrections,
            allow_negative_plx,
            must_load,
            additional,
            indices,
            coord: coord::Coord::new(),
            counts_per_mag: RefCell::new([0; 22]),
        }
    }

    pub fn load_dir(&self, dir: &str) -> Result<Vec<Particle>, &str> {
        let mut list: Vec<Particle> = Vec::new();
        let mut i = 0;

        if Path::new(dir).is_file() {
            self.load_file(dir, &mut list, 1, 1);
        } else {
            // glob directory
            let mut dir_glob: String = String::from(dir);
            dir_glob.push_str("/*");
            let dir_count = glob(&dir_glob).expect("Error reading glob pattern").count();
            let count = if self.max_files > 0 {
                usize::min(self.max_files as usize, dir_count)
            } else {
                dir_count
            };
            for entry in glob(&dir_glob).expect("Error reading glob pattern") {
                if self.max_files >= 0 && i >= self.max_files {
                    return Ok(list);
                }
                match entry {
                    Ok(path) => {
                        self.load_file(
                            path.to_str().expect("Error: path not valid"),
                            &mut list,
                            (i + 1) as usize,
                            count,
                        );
                    }
                    Err(e) => log::error!("Error: {:?}", e),
                }
                i += 1;
            }
        }
        Ok(list)
    }

    // Loads a single file, being it csv.gz or csv
    // The format is hard-coded for now, with csv.gz being in eDR3 format,
    // and csv being in hipparcos format.
    pub fn load_file(
        &self,
        file: &str,
        list: &mut Vec<Particle>,
        file_num: usize,
        file_count: usize,
    ) {
        let mut total: usize = 0;
        let mut loaded: usize = 0;
        let mut skipped: usize = 0;
        let is_gz = file.ends_with(".gz") || file.ends_with(".gzip");
        let f = File::open(file).expect("Error: file not found");
        let mmap = unsafe { Mmap::map(&f).expect(&format!("Error mapping file {}", file)) };

        let mut reader: Box<dyn Read>;
        if is_gz {
            reader = Box::new(GzDecoder::new(&mmap[..]));
        } else {
            reader = Box::new(&mmap[..]);
        }

        for line in io::BufReader::new(reader.as_mut()).lines() {
            // Skip header
            if total > 0 {
                match self.parse_line(line.expect("Error reading line")) {
                    Some(part) => {
                        list.push(part);
                        loaded += 1;
                    }
                    None => skipped += 1,
                }
            }
            total += 1;
            if self.max_records >= 0 && (total - 1) as i32 >= self.max_records {
                break;
            }
        }
        self.log_file(loaded, total, skipped, file, file_num, file_count);
    }

    fn log_file(
        &self,
        loaded: usize,
        total: usize,
        skipped: usize,
        file: &str,
        file_num: usize,
        file_count: usize,
    ) {
        log::info!(
            "{}/{} ({:.3}%): {} -> {}/{} stars ({:.3}%, {} skipped)",
            file_num,
            file_count,
            100.0 * file_num as f32 / file_count as f32,
            Path::new(file).file_name().unwrap().to_str().unwrap(),
            loaded,
            total - 1,
            100.0 * loaded as f32 / (total as f32 - 1.0),
            skipped
        );
    }

    fn get_index(&self, col_id: &ColId) -> usize {
        match self.indices.get(col_id) {
            Some(value) => *value,
            // Set out of range so that tokens.get() produces None
            None => 5000,
        }
    }

    // Parses a line using self.indices
    fn parse_line(&self, line: String) -> Option<Particle> {
        let sep = Regex::new(r"\s+|,").unwrap();
        let tokens: Vec<&str> = sep.split(&line).collect();

        self.create_particle(
            tokens.get(self.get_index(&ColId::source_id)),
            tokens.get(self.get_index(&ColId::hip)),
            tokens.get(self.get_index(&ColId::names)),
            tokens.get(self.get_index(&ColId::ra)),
            tokens.get(self.get_index(&ColId::dec)),
            tokens.get(self.get_index(&ColId::plx)),
            tokens.get(self.get_index(&ColId::plx_err)),
            tokens.get(self.get_index(&ColId::pmra)),
            tokens.get(self.get_index(&ColId::pmdec)),
            tokens.get(self.get_index(&ColId::radvel)),
            tokens.get(self.get_index(&ColId::gmag)),
            tokens.get(self.get_index(&ColId::bpmag)),
            tokens.get(self.get_index(&ColId::rpmag)),
            tokens.get(self.get_index(&ColId::col_idx)),
            tokens.get(self.get_index(&ColId::ruwe)),
            tokens.get(self.get_index(&ColId::ag)),
            tokens.get(self.get_index(&ColId::ebp_min_rp)),
            tokens.get(self.get_index(&ColId::teff)),
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
        snames: Option<&&str>,
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
        sag: Option<&&str>,
        sebp_min_rp: Option<&&str>,
        steff: Option<&&str>,
    ) -> Option<Particle> {
        // First, check if we accept it given the current constraints
        let mut plx: f64 = parse::parse_f64(splx) + self.plx_zeropoint;
        let plx_e: f64 = parse::parse_f64(splx_e);
        let mut appmag: f64 = parse::parse_f64(sappmag);

        let has_geodist = self.has_additional_col(ColId::geodist);

        // Source ID
        let mut source_id: i64 = parse::parse_i64(ssource_id);
        let hip: i32 = parse::parse_i32(ship_id);
        if source_id == 0 {
            source_id = hip as i64;
        }

        let must_load = self.must_load_particle(source_id);

        // Parallax test
        if !has_geodist && plx <= 0.0 && self.allow_negative_plx {
            plx = 0.04;
        } else if !must_load && !self.accept_parallax(has_geodist, appmag, plx, plx_e) {
            return None;
        }

        // Extra attributes
        let mut extra: HashMap<ColId, f32> = HashMap::with_capacity(2);

        let ruwe_val: f32 = self.get_ruwe(source_id, sruwe);
        // RUWE test
        if !must_load && !self.accept_ruwe(ruwe_val) {
            return None;
        }
        if ruwe_val.is_finite() {
            extra.insert(ColId::ruwe, ruwe_val);
        }

        // If we have geometric distances, we only accept stars which have one, otherwise
        // we accept all
        if !(must_load
            || !has_geodist
            || (has_geodist && self.has_additional(ColId::geodist, source_id)))
        {
            return None;
        }

        // Distance
        let mut dist_pc: f64 = 1000.0 / plx;
        let geodist_pc = self.get_geodistance(source_id);
        dist_pc = if geodist_pc > 0.0 {
            geodist_pc
        } else {
            dist_pc
        };

        // Distance test
        if !must_load && !self.accept_distance(dist_pc) {
            return None;
        }
        let dist: f64 = dist_pc * constants::PC_TO_U;

        // Parallax error
        let plx_err: f32 = parse::parse_f32(splx_e);
        if plx_err.is_finite() {
            extra.insert(ColId::plx_err, plx_err);
        }

        // Name
        let mut name_vec = Vec::new();
        if snames.is_some() {
            let name_tokens: Vec<&str> = snames.unwrap().split("|").collect();
            for name_token in name_tokens {
                name_vec.push(String::from(name_token));
            }
        }

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
        let mut ag: f64 = parse::parse_f64(sag);
        let pos_gal: Vector3<f64> = Vector3::new(pos.x, pos.y, pos.z);
        self.coord.eq_gal.transform_vector(&pos_gal);
        let pos_gal_sph = util::cartesian_to_spherical(pos_gal.x, pos_gal.y, pos_gal.z);
        let b = pos_gal_sph.y;
        let magcorraux = dist_pc.min(150.0 / b.sin().abs());

        if self.mag_corrections {
            if !ag.is_finite() {
                // Analytical extinction, cap to 3.2
                let ag_analytical: f64 = f64::min(3.2, magcorraux * 5.9e-4);
                if self.has_additional_col(ColId::ag) {
                    ag = self
                        .get_additional(ColId::ag, source_id)
                        .unwrap_or(ag_analytical);
                } else {
                    ag = ag_analytical;
                }
            }
        }
        if ag.is_finite() {
            appmag -= ag;
        }

        // Absolute magnitude
        let absmag = appmag - 5.0 * (if dist_pc <= 0.0 { 10.0 } else { dist_pc }).log10() + 5.0;

        // Size
        let pseudo_l = 10.0_f64.powf(-0.4 * absmag);
        let size_fac = constants::PC_TO_M * constants::M_TO_U * 0.15;
        let size: f32 = f64::min(pseudo_l.powf(0.45) * size_fac, 1e10) as f32;

        // Color
        let mut ebr: f64 = parse::parse_f64(sebp_min_rp);
        if self.mag_corrections {
            if !self.has_col(ColId::ebp_min_rp) || ebr.is_finite() {
                // Analytical reddening, cap to 1.6
                let ebr_analytical = f64::min(1.6, magcorraux * 2.9e-4);
                if self.has_additional(ColId::ebp_min_rp, source_id) {
                    ebr = self
                        .get_additional(ColId::ebp_min_rp, source_id)
                        .unwrap_or(ebr_analytical);
                } else {
                    ebr = ebr_analytical;
                }
            }
        }

        let col_idx: f64;
        let mut teff: f64 = parse::parse_f64(steff);
        if sbp.is_some() && srp.is_some() {
            let bp: f64 = parse::parse_f64(sbp);
            let rp: f64 = parse::parse_f64(srp);
            col_idx = bp - rp - ebr;

            if !teff.is_finite() {
                teff = color::xp_to_teff(col_idx);
            }
        } else if sbv.is_some() {
            col_idx = parse::parse_f64(sbv);
            teff = color::bv_to_teff_ballesteros(col_idx);
        } else {
            col_idx = 0.656;
            teff = color::bv_to_teff_ballesteros(col_idx);
        }
        let (col_r, col_g, col_b) = color::teff_to_rgb(teff);
        let color_packed: f32 = color::col_to_f32(col_r as f32, col_g as f32, col_b as f32, 1.0);

        // Update counts per mag
        let appmag_clamp = f64::clamp(appmag, 0.0, 21.0) as usize;
        self.counts_per_mag.borrow_mut()[appmag_clamp] += 1;

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
            col: color_packed,
            size,
            hip,
            id: source_id,
            names: name_vec,
            extra,
        })
    }

    fn accept_parallax(&self, has_geodist: bool, appmag: f64, plx: f64, plx_e: f64) -> bool {
        // If geometric distances are present, always accept, we use distances directly regardless of parallax
        if has_geodist {
            return true;
        }

        if !appmag.is_finite() {
            return false;
        } else if appmag < 13.1 {
            return plx >= 0.0 && plx_e < plx * self.plx_err_bright && plx_e < self.plx_err_cap;
        } else {
            return plx >= 0.0 && plx_e < plx * self.plx_err_faint && plx_e < self.plx_err_cap;
        }
    }

    fn accept_distance(&self, dist_pc: f64) -> bool {
        dist_pc <= self.distpc_cap
    }

    fn get_geodistance(&self, source_id: i64) -> f64 {
        let geodist = self.get_additional(ColId::geodist, source_id);
        match geodist {
            Some(d) => {
                if d.is_finite() {
                    d
                } else {
                    -1.0
                }
            }
            None => -1.0,
        }
    }

    fn accept_ruwe(&self, ruwe: f32) -> bool {
        ruwe.is_nan() || self.ruwe_cap.is_nan() || ruwe < self.ruwe_cap
    }

    fn get_ruwe(&self, source_id: i64, sruwe: Option<&&str>) -> f32 {
        if self.has_col(ColId::ruwe) {
            parse::parse_f32(sruwe)
        } else {
            let ruwe = self.get_additional(ColId::ruwe, source_id);
            match ruwe {
                Some(d) => {
                    if d.is_finite() {
                        d as f32
                    } else {
                        f32::NAN
                    }
                }
                None => f32::NAN,
            }
        }
    }

    fn get_additional(&self, col_id: ColId, source_id: i64) -> Option<f64> {
        if self.additional.is_empty() {
            None
        } else {
            for entry in &self.additional {
                if entry.has_col(col_id) {
                    let d = entry.get(col_id, source_id);
                    match d {
                        Some(val) => return Some(val),
                        None => return None,
                    }
                }
            }
            None
        }
    }

    fn has_col(&self, col_id: ColId) -> bool {
        !self.indices.is_empty()
            && self.indices.contains_key(&col_id)
            && self.indices.get(&col_id).expect("Error getting column") >= &0
    }

    fn has_additional_col(&self, col_id: ColId) -> bool {
        if self.additional.is_empty() {
            false
        } else {
            for entry in &self.additional {
                if entry.has_col(col_id) {
                    return true;
                }
            }
            false
        }
    }

    fn has_additional(&self, col_id: ColId, source_id: i64) -> bool {
        let val = self.get_additional(col_id, source_id);
        match val {
            Some(v) => !v.is_nan() && !v.is_finite(),
            None => false,
        }
    }
}
