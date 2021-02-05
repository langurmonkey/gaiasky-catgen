use crate::load;

use std::collections::HashMap;

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
    pub col: u32,
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

impl Eq for Vec3 {}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Vec3 {
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
    pub ruwe_cap: f32,
    pub distpc_cap: f64,
    pub plx_err_faint: f64,
    pub plx_err_bright: f64,
    pub plx_zeropoint: f64,
    pub mag_corrections: bool,
    pub postprocess: bool,
    pub child_count: usize,
    pub parent_count: usize,
    pub additional: String,
    pub xmatch: String,
    pub columns: String,
}

/**
 * A map with i64 keys backed by a number of
 * regular HashMaps. It is supposed to hold
 * very many entries.
 **/
pub struct LargeLongMap<T> {
    pub n_maps: u32,
    pub maps: Vec<HashMap<i64, T>>,
    pub empty: bool,
    pub size: usize,
}

impl<T> LargeLongMap<T> {
    pub fn new(n: u32) -> LargeLongMap<T> {
        let mut llm = LargeLongMap::<T> {
            n_maps: n,
            empty: true,
            maps: Vec::new(),
            size: 0,
        };
        for i in 0..n {
            llm.maps.push(HashMap::new());
        }
        llm
    }

    pub fn contains_key(&self, key: i64) -> bool {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps
            .get(idx)
            .expect("Error: map not found")
            .contains_key(&key)
    }

    pub fn put(&mut self, key: i64, value: T) {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps
            .get_mut(idx)
            .expect("Error: map not found")
            .insert(key, value);
        self.empty = false;
        self.size += 1;
    }

    pub fn get(&self, key: i64) -> Option<&T> {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps.get(idx).expect("Error: map not found").get(&key)
    }

    pub fn get_mut(&mut self, key: i64) -> Option<&mut T> {
        let idx: usize = (key % self.n_maps as i64) as usize;
        self.maps
            .get_mut(idx)
            .expect("Error: map not found")
            .get_mut(&key)
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }
}
