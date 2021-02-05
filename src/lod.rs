use crate::constants;
use crate::data;

use data::Particle;
use data::Vec3;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::{borrow::Borrow, time::Instant};
use std::{borrow::BorrowMut, collections::HashMap};

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
     * number of stars actually added (i.e. not skipped
     * due to being too far)
     **/
    pub fn generate_octree(&self, list: &Vec<Particle>) -> usize {
        self.start_generation(list);

        let mut octree_star_num: usize = 0;
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
                let star_idx = cat_idx;
                cat_idx += 1;

                // Check if star is too far
                let dist = (star.x * star.x + star.y * star.y + star.z * star.z).sqrt();
                if dist * constants::U_TO_PC > self.distpc_cap {
                    continue;
                }

                let x = star.x;
                let y = star.y;
                let z = star.z;

                let mut octant_id: OctantId;
                if !self.has_node(x, y, z, level) {
                    // Octree node does not exist yet, create it
                    octant_id = self.create_octant(x, y, z, level);
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
                octree_star_num += 1;

                if added_num >= self.max_part {
                    // Next level
                    break;
                }
            }

            if (cat_idx >= list.len()) {
                // All stars added -> FINISHED
                break;
            }
        }
        octree_star_num
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
        for l in 1..level {
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

        let mut max_dist = -1.0;
        let mut furthest = list.get(0).unwrap();

        for particle in list {
            let dist =
                (particle.x * particle.x + particle.y * particle.y + particle.z * particle.z)
                    .sqrt();
            if dist * constants::U_TO_PC <= self.distpc_cap {
                if dist > max_dist {
                    max_dist = dist;
                    furthest = &particle;
                }
            }
        }
        println!("Furthest star is at {} pc", max_dist * constants::U_TO_PC);

        let pos1: data::Vec3 = Vec3 {
            x: furthest.x,
            y: furthest.y,
            z: furthest.z,
        };

        let mut volume = -1.0;
        let mut pos0: data::Vec3;

        let mut min: data::Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut max: data::Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        for particle in list {
            pos0 = Vec3 {
                x: particle.x,
                y: particle.y,
                z: particle.z,
            };
            let vol = compute_volume(&pos1, &pos0);
            if vol > volume {
                volume = vol;
                min = pos1.copy();
                max = pos0.copy();
            }
        }
        let half_size = (max.x - min.x) / 2.0;

        let root = Octant {
            id: OctantId(0),
            min: min.copy(),
            max: max.copy(),
            centre: Vec3 {
                x: (min.x + max.x) / 2.0,
                y: (min.y + max.y) / 2.0,
                z: (min.z + max.z) / 2.0,
            },
            size: Vec3 {
                x: half_size,
                y: half_size,
                z: half_size,
            },
            level: 0,

            parent: None,
            children: RefCell::new([None; 8]),
            objects: RefCell::new(Vec::new()),
        };
        println!(
            "Octree generation started with min: {:?}, max: {:?}, centre: {:?}",
            root.min, root.max, root.centre
        );
        self.nodes.borrow_mut().push(root);
    }

    pub fn get_num_objects(&self) -> usize {
        self.nodes.borrow()[0].get_num_objects(self)
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
            "{}{}:L{} ID: {} Objs:{}",
            String::from_utf8(vec![32; self.level as usize]).unwrap(),
            parent_idx,
            self.level,
            self.id.0,
            self.objects.borrow().len()
        );

        if self.has_kids() {
            for i in 0..8 {
                let b = self.children.borrow();
                let c = b[i];
                match c {
                    Some(child) => octree.nodes.borrow().get(child.0).unwrap().print(i, octree),
                    None => println!(
                        "{}{}:L{} x",
                        String::from_utf8(vec![32; (self.level + 1) as usize]).unwrap(),
                        i,
                        self.level + 1
                    ),
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

    pub fn get_num_objects(&self, octree: &Octree) -> usize {
        let mut num = self.objects.borrow().len();
        if self.has_kids() {
            for i in 0..8 {
                if self.children.borrow()[i].is_some() {
                    let idx = self.children.borrow()[i].unwrap();
                    num += octree.nodes.borrow()[idx.0].get_num_objects(octree);
                }
            }
        }
        num
    }
}

/**
 * Computes the volume of the axis-aligned box
 * within the points min and max
 **/
fn compute_volume(min: &Vec3, max: &Vec3) -> f64 {
    let dim = Vec3 {
        x: max.x - min.x,
        y: max.y - min.y,
        z: max.z - min.z,
    };
    dim.x * dim.y * dim.z
}
