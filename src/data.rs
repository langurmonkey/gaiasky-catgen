use crate::constants;
use crate::load;

use std::collections::HashMap;
use std::fmt;

/**
 * Represents a star. The cartesian
 * positions use double-precision floating point
 * numbers. The rest use single-precision.
 **/
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pmx: f32,
    pub pmy: f32,
    pub pmz: f32,
    pub mualpha: f32,
    pub mudelta: f32,
    pub radvel: f32,
    pub appmag: f32,
    pub absmag: f32,
    pub col: f32,
    pub size: f32,
    pub hip: i32,
    pub id: i64,
    pub names: Vec<String>,
    pub extra: HashMap<load::ColId, f32>,
}

impl Particle {
    pub fn get_extra(&self, col_id: load::ColId) -> &f32 {
        match self.extra.get(&col_id) {
            Some(val) => &val,
            None => &f32::NAN,
        }
    }

    pub fn copy(&self) -> Self {
        // Copy names
        let mut new_names = Vec::new();
        for name in &self.names {
            new_names.push(String::from(name));
        }
        // Copy extra
        let mut new_extra = HashMap::new();
        for key in self.extra.keys() {
            new_extra.insert(*key, *self.extra.get(key).unwrap());
        }
        Particle {
            x: self.x,
            y: self.y,
            z: self.z,
            pmx: self.pmx,
            pmy: self.pmy,
            pmz: self.pmz,
            mualpha: self.mualpha,
            mudelta: self.mudelta,
            radvel: self.radvel,
            appmag: self.appmag,
            absmag: self.absmag,
            col: self.col,
            size: self.size,
            hip: self.hip,
            id: self.id,
            names: new_names,
            extra: new_extra,
        }
    }
}

/**
 * Simple vector with three components
 **/
#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{:.3}, {:.3}, {:.3}]",
            self.x * constants::U_TO_PC,
            self.y * constants::U_TO_PC,
            self.z * constants::U_TO_PC
        )
    }
}

impl Eq for Vec3 {}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Vec3 {
    pub fn empty() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }
    pub fn with(value: f64) -> Vec3 {
        Vec3 {
            x: value,
            y: value,
            z: value,
        }
    }
    pub fn copy(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn set(&mut self, x: f64, y: f64, z: f64) -> &Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn set_from(&mut self, other: &Self) -> &Self {
        self.x = other.x;
        self.y = other.y;
        self.z = other.z;
        self
    }

    pub fn len(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }
}

/**
 * An axis-aligned bounding box
 **/
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,

    pub cnt: Vec3,
    pub dim: Vec3,
}

impl BoundingBox {
    pub fn empty() -> BoundingBox {
        BoundingBox {
            min: Vec3::empty(),
            max: Vec3::empty(),
            cnt: Vec3::empty(),
            dim: Vec3::empty(),
        }
    }

    pub fn from(min: &Vec3, max: &Vec3) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        bb.set_min_max(min, max);
        bb
    }

    pub fn set_min_max(&mut self, min: &Vec3, max: &Vec3) {
        let minx = if min.x < max.x { min.x } else { max.x };
        let miny = if min.y < max.y { min.y } else { max.y };
        let minz = if min.z < max.z { min.z } else { max.z };
        let maxx = if min.x > max.x { min.x } else { max.x };
        let maxy = if min.y > max.y { min.y } else { max.y };
        let maxz = if min.z > max.z { min.z } else { max.z };
        self.min.set(minx, miny, minz);
        self.max.set(maxx, maxy, maxz);
        self.cnt.set(
            (minx + maxx) / 2.0,
            (miny + maxy) / 2.0,
            (minz + maxz) / 2.0,
        );
        self.dim.set(maxx - minx, maxy - miny, maxz - minz);
    }

    #[allow(dead_code)]
    pub fn set(&mut self, other: &BoundingBox) {
        self.min.set_from(&other.min);
        self.max.set_from(&other.max);
        self.cnt.set_from(&other.cnt);
        self.dim.set_from(&other.dim);
    }

    #[allow(dead_code)]
    pub fn volume(&self) -> f64 {
        self.dim.x * self.dim.y * self.dim.z
    }

    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.min.x < self.max.x && self.min.y < self.max.y && self.min.z < self.max.z
    }

    #[allow(dead_code)]
    pub fn contains_vec(&self, vec: &Vec3) -> bool {
        self.contains(vec.x, vec.y, vec.z)
    }

    #[allow(dead_code)]
    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        self.min.x <= x
            && self.max.x >= x
            && self.min.y <= y
            && self.max.y >= y
            && self.min.z <= z
            && self.max.z >= z
    }
}

/**
 * Holds the program configuration, which
 * corresponds roughly to the CLI arguments
 **/
pub struct Config {
    pub input: String,
    pub output: String,
    pub hip: String,
    pub max_part: usize,
    // limit ruwe value.
    pub ruwe_cap: f32,
    // limit distance in parsecs.
    pub distpc_cap: f64,
    // ignore parallax cuts and use all stars with photometric distances.
    pub photdist: bool,
    // parallax error threshold for faint stars (gmag >= 13.1), where plx_err/plx < plx_err_faint.
    pub plx_err_faint: f64,
    // parallax error threshold for bright stars (gmag < 13.1), where plx_err/plx < plx_err_bright.
    pub plx_err_bright: f64,
    pub plx_zeropoint: f64,
    pub mag_corrections: u8,
    pub allow_negative_plx: bool,
    // Put the centre of the octree at the reference system origin (0 0 0).
    pub centre_origin: bool,
    // post-process the octree to try to flatten it.
    pub postprocess: bool,
    pub child_count: usize,
    pub parent_count: usize,
    pub additional: String,
    pub xmatch: String,
    pub columns: String,
    pub file_num_cap: i32,
    pub star_num_cap: i32,
    pub dry_run: bool,
    pub debug: bool,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arguments")
            .field("input", &self.input)
            .field("output", &self.output)
            .field("hip", &self.hip)
            .field("max_part", &self.max_part)
            .field("ruwe_cap", &self.ruwe_cap)
            .field("distpc_cap", &self.distpc_cap)
            .field("plx_err_faint", &self.plx_err_faint)
            .field("plx_err_bright", &self.plx_err_bright)
            .field("plx_zeropoint", &self.plx_zeropoint)
            .field("mag_corrections", &self.mag_corrections)
            .field("allow_negative_plx", &self.allow_negative_plx)
            .field("photdist", &self.photdist)
            .field("centre_origin", &self.centre_origin)
            .field("postprocess", &self.postprocess)
            .field("child_count", &self.child_count)
            .field("parent_count", &self.parent_count)
            .field("additional", &self.additional)
            .field("xmatch", &self.xmatch)
            .field("columns", &self.columns)
            .field("file_num_cap", &self.file_num_cap)
            .field("star_num_cap", &self.star_num_cap)
            .field("dry_run", &self.dry_run)
            .field("debug", &self.debug)
            .finish()
    }
}

/**
 * A map with i64 keys backed by a number of
 * regular HashMaps. It is supposed to hold
 * very many entries.
 **/
#[allow(dead_code)]
pub struct LargeLongMap<T> {
    pub n_maps: u32,
    pub maps: Vec<HashMap<i64, T>>,
    pub empty: bool,
    pub size: usize,
}

impl<T> LargeLongMap<T> {
    #[allow(dead_code)]
    pub fn new(n: u32) -> LargeLongMap<T> {
        let mut llm = LargeLongMap::<T> {
            n_maps: n,
            empty: true,
            maps: Vec::new(),
            size: 0,
        };
        for _ in 0..n {
            llm.maps.push(HashMap::new());
        }
        llm
    }

    #[allow(dead_code)]
    pub fn contains_key(&self, key: i64) -> bool {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps
            .get(idx)
            .expect("Error: map not found")
            .contains_key(&key)
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, key: i64, value: T) {
        self.put(key, value)
    }

    #[allow(dead_code)]
    pub fn put(&mut self, key: i64, value: T) {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps
            .get_mut(idx)
            .expect("Error: map not found")
            .insert(key, value);
        self.empty = false;
        self.size += 1;
    }

    #[allow(dead_code)]
    pub fn get(&self, key: i64) -> Option<&T> {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps.get(idx).expect("Error: map not found").get(&key)
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, key: i64) -> Option<&mut T> {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps
            .get_mut(idx)
            .expect("Error: map not found")
            .get_mut(&key)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.empty
    }
}
