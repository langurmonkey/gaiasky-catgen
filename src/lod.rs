use crate::constants;
use crate::data;

use data::{BoundingBox, Particle, Vec3};
use std::cell::{Cell, RefCell};
use std::fmt;

/**
 * An octree contains a root
 * and a map
 **/
pub struct Octree {
    pub max_part: usize,
    pub postprocess: bool,
    pub child_count: usize,
    pub parent_count: usize,
    pub distpc_cap: f64,

    // private data with all octants in the octree
    // the id of each octant is the index in this list
    pub nodes: RefCell<Vec<Octant>>,
    // Root node index
    pub root: Option<OctantId>,
}

impl Octree {
    /**
     * Creates a new empty octree with the given parameters and no nodes
     **/
    pub fn from_params(
        max_part: usize,
        postprocess: bool,
        child_count: usize,
        parent_count: usize,
        distpc_cap: f64,
    ) -> Self {
        Octree {
            max_part,
            postprocess,
            child_count,
            parent_count,
            distpc_cap,

            nodes: RefCell::new(Vec::new()),
            root: None,
        }
    }

    /**
     * Generates a new octree with the given list
     * of stars and parameters. The list must be
     * sorted by magnitude beforehand. Returns the
     * number of octants in the octree, the number
     * of stars actually added (i.e. not skipped
     * due to being too far) and the depth of the tree
     **/
    pub fn generate_octree(&self, list: &Vec<Particle>) -> (usize, usize, u32) {
        self.start_generation(list);

        let mut octree_star_num: usize = 0;
        let mut octree_node_num: usize = 1;
        let mut depth: u32 = 0;
        let mut cat_idx = 0;
        for level in 0..25 {
            println!(
                "Generating level {} ({} stars left)",
                level,
                list.len() - cat_idx
            );
            while cat_idx < list.len() {
                // Add stars to nodes until we reach max_part
                let star = list.get(cat_idx).expect("Error getting star");

                // Check if star is too far
                let dist = (star.x * star.x + star.y * star.y + star.z * star.z).sqrt();
                if dist * constants::U_TO_PC > self.distpc_cap {
                    println!(
                        "Star {} discarded due to being too far ({} pc)",
                        star.id,
                        dist * constants::U_TO_PC
                    );
                    cat_idx += 1;
                    continue;
                }

                let x = star.x;
                let y = star.y;
                let z = star.z;

                let octant_id: OctantId;

                if !self.has_node(x, y, z, level) {
                    // Octree node does not exist yet, create it
                    octant_id = self.create_octant(x, y, z, level);
                    octree_node_num += 1;
                } else {
                    octant_id = self.get_node(x, y, z, level).expect("Node does not exist!");
                }

                // Add star to octant
                let added_num = self
                    .nodes
                    .borrow()
                    .get(octant_id.0)
                    .unwrap()
                    .add_obj(cat_idx);

                if level > depth {
                    depth = level;
                }
                octree_star_num += 1;
                cat_idx += 1;

                if added_num >= self.max_part {
                    // Next level
                    break;
                }
            }

            if cat_idx >= list.len() {
                // All stars added -> FINISHED
                break;
            }
        }

        // Post-process
        if self.postprocess {
            todo!();
        }

        // Compute numbers
        self.nodes.borrow()[0].compute_numbers(&self);
        (octree_node_num, octree_star_num, depth)
    }

    fn has_node(&self, x: f64, y: f64, z: f64, level: u32) -> bool {
        for node in self.nodes.borrow().iter() {
            if node.level == level && node.contains(x, y, z) {
                return true;
            }
        }
        false
    }

    fn get_node(&self, x: f64, y: f64, z: f64, level: u32) -> Option<OctantId> {
        for node in self.nodes.borrow().iter() {
            if node.level == level && node.contains(x, y, z) {
                return Some(OctantId(node.id.0));
            }
        }
        None
    }

    fn add_new_node(
        &self,
        x: f64,
        y: f64,
        z: f64,
        half_size: f64,
        level: u32,
        parent: Option<OctantId>,
    ) -> OctantId {
        let new_id = self.nodes.borrow().len();
        let octant = Octant::from_params(new_id, x, y, z, half_size, level, parent);
        self.nodes.borrow_mut().push(octant);
        OctantId(new_id)
    }

    /**
     * Creates a new octant with the given parameters
     **/
    pub fn create_octant(&self, x: f64, y: f64, z: f64, level: u32) -> OctantId {
        let mut min: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        // start at root, which is always 0
        let mut current = OctantId(0);
        for l in 1..=level {
            let hs: f64 = self.nodes.borrow().get(current.0).unwrap().size.x / 2.0;
            let idx;

            let cmin = self.nodes.borrow().get(current.0).unwrap().min;

            if x <= cmin.x + hs {
                if y <= cmin.y + hs {
                    if z <= cmin.z + hs {
                        idx = 0;
                        min.set_from(&cmin);
                    } else {
                        idx = 1;
                        min.set(cmin.x, cmin.y, cmin.z + hs);
                    }
                } else {
                    if z <= cmin.z + hs {
                        idx = 2;
                        min.set(cmin.x, cmin.y + hs, cmin.z);
                    } else {
                        idx = 3;
                        min.set(cmin.x, cmin.y + hs, cmin.z + hs);
                    }
                }
            } else {
                if y <= cmin.y + hs {
                    if z <= cmin.z + hs {
                        idx = 4;
                        min.set(cmin.x + hs, cmin.y, cmin.z);
                    } else {
                        idx = 5;
                        min.set(cmin.x + hs, cmin.y, cmin.z + hs);
                    }
                } else {
                    if z <= cmin.z + hs {
                        idx = 6;
                        min.set(cmin.x + hs, cmin.y + hs, cmin.z);
                    } else {
                        idx = 7;
                        min.set(cmin.x + hs, cmin.y + hs, cmin.z + hs);
                    }
                }
            }

            if !self.nodes.borrow().get(current.0).unwrap().has_child(idx) {
                // Create kid
                let nhs: f64 = hs / 2.0;

                let x = min.x + nhs;
                let y = min.y + nhs;
                let z = min.z + nhs;
                let hs = nhs;

                let node_id = self.add_new_node(x, y, z, hs, l, Some(current));
                self.nodes
                    .borrow()
                    .get(current.0)
                    .unwrap()
                    .add_child(idx, node_id);
            }
            current = self
                .nodes
                .borrow()
                .get(current.0)
                .unwrap()
                .get_child(idx)
                .expect("OctantId does not exist!");
        }

        current
    }

    /**
     * Computes the maximum axis-aligned bounding box
     * containing all the particles in the list.
     **/
    fn start_generation(&self, list: &Vec<Particle>) {
        println!("Starting generation of octree");

        let mut min = Vec3::with(1.0e50);
        let mut max = Vec3::with(-1.0e50);

        for particle in list {
            if particle.x < min.x {
                min.x = particle.x;
            }
            if particle.y < min.y {
                min.y = particle.y;
            }
            if particle.z < min.z {
                min.z = particle.z;
            }
            if particle.x > max.x {
                max.x = particle.x;
            }
            if particle.y > max.y {
                max.y = particle.y;
            }
            if particle.z > max.z {
                max.z = particle.z;
            }
        }
        // The bounding box
        let bx = BoundingBox::from(&min, &max);
        let half_size = f64::max(f64::max(bx.dim.z, bx.dim.y), bx.dim.x) / 2.0;

        let root = Octant {
            id: OctantId(0),
            min: bx.min.copy(),
            max: bx.max.copy(),
            centre: bx.cnt.copy(),
            size: Vec3 {
                x: half_size,
                y: half_size,
                z: half_size,
            },
            level: 0,

            num_objects: Cell::new(0),
            num_objects_rec: Cell::new(0),
            num_children: Cell::new(0),

            parent: None,
            children: RefCell::new([None; 8]),
            objects: RefCell::new(Vec::new()),
        };

        // Volume of root node in pc^3
        let vol = bx.dim.x
            * constants::U_TO_PC
            * bx.dim.y
            * constants::U_TO_PC
            * bx.dim.z
            * constants::U_TO_PC;
        println!(
            "Octree root node generated with min: {}, max: {}, centre: {}, volume: {} pc3",
            root.min, root.max, root.centre, vol
        );
        self.nodes.borrow_mut().push(root);
    }

    pub fn get_num_objects(&self) -> i32 {
        self.nodes.borrow()[0].num_objects_rec.get()
    }

    pub fn print(&self) {
        println!(
            "Octree contains {} nodes and {} objects",
            self.nodes.borrow().len(),
            self.get_num_objects()
        );

        // Print root
        self.nodes
            .borrow()
            .get(0)
            .expect("Octree has no root")
            .print(0, self);
    }
}

impl fmt::Debug for Octree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Octree")
            .field("n_nodes", &self.nodes.borrow().len())
            .finish()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Default, Hash)]
pub struct OctantId(pub usize);

/**
 * Defines an octree, a tree in which
 * each node is a cube and parts the
 * space into 8 children sub-cubes
 **/
#[derive(Eq, PartialEq)]
pub struct Octant {
    pub id: OctantId,
    pub min: Vec3,
    pub max: Vec3,
    pub centre: Vec3,
    pub size: Vec3,
    pub level: u32,

    pub num_objects: Cell<i32>,
    pub num_objects_rec: Cell<i32>,
    pub num_children: Cell<i32>,

    pub parent: Option<OctantId>,
    pub children: RefCell<[Option<OctantId>; 8]>,
    pub objects: RefCell<Vec<usize>>,
}

impl Octant {
    /**
     * Creates a shallow octree node with the given centre, half size and depth
     **/
    pub fn from_params(
        id: usize,
        x: f64,
        y: f64,
        z: f64,
        half_size: f64,
        level: u32,
        parent: Option<OctantId>,
    ) -> Self {
        Octant {
            id: OctantId(id),
            min: Vec3 {
                x: x - half_size,
                y: y - half_size,
                z: z - half_size,
            },
            max: Vec3 {
                x: x + half_size,
                y: y + half_size,
                z: z + half_size,
            },
            centre: Vec3 { x, y, z },
            size: Vec3 {
                x: half_size * 2.0,
                y: half_size * 2.0,
                z: half_size * 2.0,
            },
            level,

            num_objects: Cell::new(0),
            num_objects_rec: Cell::new(0),
            num_children: Cell::new(0),

            parent,
            children: RefCell::new([None; 8]),
            objects: RefCell::new(Vec::new()),
        }
    }

    pub fn add_child(&self, index: usize, node: OctantId) {
        self.children.borrow_mut()[index] = Some(node);
    }

    pub fn add_obj(&self, index: usize) -> usize {
        self.objects.borrow_mut().push(index);
        self.objects.borrow().len()
    }

    pub fn has_child(&self, index: usize) -> bool {
        self.children.borrow()[index].is_some()
    }

    pub fn get_child(&self, index: usize) -> Option<OctantId> {
        self.children.borrow()[index]
    }

    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        self.min.x <= x
            && self.max.x >= x
            && self.min.y <= y
            && self.max.y >= y
            && self.min.z <= z
            && self.max.z >= z
    }

    pub fn print(&self, parent_idx: usize, octree: &Octree) {
        // 32 is the UTF-8 code for whitespace
        println!(
            "{}{}:L{} id:{} Obj(own/rec):({}/{}) Nchld:{}",
            String::from_utf8(vec![32; (self.level * 2) as usize]).unwrap(),
            parent_idx,
            self.level,
            self.id.0,
            self.num_objects.get(),
            self.num_objects_rec.get(),
            self.num_children.get(),
        );

        if self.has_kids() {
            for i in 0..8 {
                let b = self.children.borrow();
                let c = b[i];
                match c {
                    Some(child) => octree.nodes.borrow().get(child.0).unwrap().print(i, octree),
                    None => (),
                }
            }
        }
    }

    fn has_kids(&self) -> bool {
        for i in 0..8 {
            if self.children.borrow()[i].is_some() {
                return true;
            }
        }
        false
    }

    /**
     * Computes the number of objects and the number of children nodes
     * recursively, sets the attributes and returns them.
     **/
    pub fn compute_numbers(&self, octree: &Octree) -> i32 {
        let mut num_objects_rec = self.get_num_objects();
        self.num_objects.set(num_objects_rec);

        let mut num_children = 0;
        for ch in 0..8 {
            if self.has_child(ch) {
                num_children += 1;
            }
        }
        self.num_children.set(num_children);

        // Recursively count objects
        for i in 0..8 {
            if self.children.borrow()[i].is_some() {
                let idx = self.children.borrow()[i].unwrap();
                let objs = octree.nodes.borrow()[idx.0].compute_numbers(octree);

                num_objects_rec += objs;
            }
        }

        self.num_objects_rec.set(num_objects_rec);

        num_objects_rec
    }

    pub fn get_num_objects(&self) -> i32 {
        self.objects.borrow().len() as i32
    }
}
