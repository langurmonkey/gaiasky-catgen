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
}
