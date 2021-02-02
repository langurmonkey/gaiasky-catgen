use std::collections::HashMap;

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
    pub names: Option<Vec<String>>,
}

pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub struct Octant {
    pub id: i64,
    pub min: Vec3<f64>,
    pub max: Vec3<f64>,
    pub centre: Vec3<f64>,
    pub size: Vec3<f64>,
    pub depth: i32,
    pub n_obj: i32,
    pub n_obj_own: i32,
    pub n_children: i32,

    pub parent: Box<Octant>,
    pub children: Vec<Octant>,
    pub objs: Vec<Particle>,
}

pub struct Args {
    pub input: String,
    pub output: String,
    pub hip: String,
    pub max_part: i32,
    pub ruwe_cap: f32,
    pub distpc_cap: f64,
    pub plx_err_faint: f64,
    pub plx_err_bright: f64,
    pub plx_zeropoint: f64,
    pub mag_corrections: bool,
    pub postprocess: bool,
    pub child_count: i32,
    pub parent_count: i32,
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
        self.maps
            .get(idx)
            .expect("Error: map not found")
            .get(&key)
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
