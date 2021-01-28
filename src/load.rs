extern crate flate2;

use crate::data;
use crate::util;
use data::Particle;
use flate2::read::GzDecoder;
use glob::glob;
use io::Write;
use std::fs::File;
use std::io;
use std::io::BufRead;

// Parsecs to internal units
const PC_TO_U: f64 = 3.08567758149137e16;

pub fn load_dir(dir: &str) -> Result<Vec<Particle>, &str> {
    let mut list: Vec<Particle> = Vec::new();
    for entry in glob(&(dir.to_owned())).expect("Error: Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                print!("Processing: {:?}", path.display());
                io::stdout().flush();
                load_file(path.to_str().expect("Error: Path not valid"), &mut list);
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
    Ok(list)
}

// Loads a single file, being it csv.gz or csv
// The format is hardcoded for now, with csv.gz being in eDR3 format,
// and csv being in hipparcos format.
pub fn load_file(file: &str, list: &mut Vec<Particle>) {
    if file.ends_with(".gz") || file.ends_with(".gzip") {
        // Read csv.gz using GzDecoder
        let f = File::open(file).expect("Error: File not found");
        let gz = GzDecoder::new(f);

        let mut total: u32 = 0;
        let mut loaded: u32 = 0;
        for line in io::BufReader::new(gz).lines() {
            // Skip header
            if total > 0 {
                let part = parse_line_gz(line.expect("Error: Read line"));
                list.push(part);
                loaded += 1;
            }
            total += 1;
        }
        println!(" - loaded {}/{} objects", loaded, total - 1);
    } else if file.ends_with(".csv") || file.ends_with(".txt") {
        // Read plain text file
        let f = File::open(file).expect("Error: File not found");

        let mut total: u32 = 0;
        let mut loaded: u32 = 0;
        for line in io::BufReader::new(f).lines() {
            // Skip header
            if total > 0 {
                let part = parse_line_csv(line.expect("Error: Read line"));
                list.push(part);
                loaded += 1;
            }
            total += 1;
        }

        println!(" - loaded {}/{} objects", loaded, total - 1);
    }
}

// Parses a hipparcos csv file and returns a Particle
// hip,name,ra,dec,plx,e_plx,pmra,pmde,mag,b_v
fn parse_line_csv(line: String) -> Particle {
    let tokens: Vec<&str> = line.split(',').collect();

    // hip
    let hip: i32 = tokens.get(0).unwrap().to_string().parse::<i32>().unwrap();

    // name
    let mut names: Vec<String> = Vec::new();
    let nms: Vec<&str> = tokens.get(1).unwrap().split("|").collect();
    for name in nms {
        names.push(name.to_string());
    }

    // position
    let ra: f64 = tokens.get(2).unwrap().to_string().parse::<f64>().unwrap();
    let dec: f64 = tokens.get(3).unwrap().to_string().parse::<f64>().unwrap();
    let plx: f64 = tokens.get(4).unwrap().to_string().parse::<f64>().unwrap();

    let dist_pc: f64 = 1000.0 / plx;
    let dist: f64 = dist_pc * PC_TO_U;

    let cartesian = util::spherical_to_cartesian(ra.to_radians(), dec.to_radians(), dist);

    let part = Particle {
        x: cartesian.x,
        y: cartesian.y,
        z: cartesian.z,
        pmx: 1.0,
        pmy: 1.0,
        pmz: 1.0,
        mualpha: 1.0,
        mudelta: 1.0,
        radvel: 1.0,
        appmag: 1.0,
        absmag: 1.0,
        col: 1.0,
        size: 1.0,
        hip: hip,
        id: 122,
        names: Some(names),
    };
    part
}

// Parses a line in eDR3 csv.gz format and returns a Particle
// sourceid,ra,dec,plx,e_ra,e_dec,e_plx,pmra,pmde,rv,gmag,bp,rp,ruwe,ref_epoch
fn parse_line_gz(line: String) -> Particle {
    let tokens: Vec<&str> = line.split(',').collect();

    // sourceid
    let sid: i64 = tokens.get(0).unwrap().to_string().parse::<i64>().unwrap();

    let part = Particle {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        pmx: 1.0,
        pmy: 1.0,
        pmz: 1.0,
        mualpha: 1.0,
        mudelta: 1.0,
        radvel: 1.0,
        appmag: 1.0,
        absmag: 1.0,
        col: 1.0,
        size: 1.0,
        hip: -1,
        id: sid,
        names: None,
    };
    part
}
