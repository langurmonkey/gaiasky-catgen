use crate::constants;
use crate::data;

use data::Particle;
use data::Vec3;
use std::{borrow::Borrow, time::Instant};
use std::{borrow::BorrowMut, collections::HashMap};

/**
 * An octree contains a root
 * and a map
 **/
pub struct Octree {
    pub max_part: i32,
    pub postprocess: bool,
    pub child_count: i32,
    pub parent_count: i32,
    // Root node index
    pub root: usize,
    // Last index in octree node list
    pub idx: usize,
    // All octree nodes
    pub nodes: Vec<OctreeNode>,
    // Octant id points to its location in node list
    pub id_map: HashMap<i64, usize>,
    // Octree node id to indices in catalog list
    pub obj_map: HashMap<i64, Vec<usize>>,
    // Octree node id to children indices in octree node list
    pub children: HashMap<i64, Vec<usize>>,
}

impl Octree {
    /**
     * Creates a new empty octree
     **/
    pub fn new(max_part: i32, postprocess: bool, child_count: i32, parent_count: i32) -> Self {
        Octree {
            max_part: max_part,
            postprocess: postprocess,
            child_count: child_count,
            parent_count: parent_count,
            root: 0,
            idx: 0,
            nodes: Vec::new(),
            id_map: HashMap::new(),
            obj_map: HashMap::new(),
            children: HashMap::new(),
        }
    }

    fn add_node(&mut self, node: OctreeNode) -> usize {
        let node_id = node.id;
        self.nodes.insert(self.idx, node);
        self.id_map.insert(node_id, self.idx);
        let result = self.idx;

        self.idx += 1;

        result
    }

    fn add_node_as_child(&mut self, node: OctreeNode, parent_id: i64, child_idx: usize) -> usize {
        let node_id = node.id;
        self.nodes.insert(self.idx, node);
        self.id_map.insert(node_id, self.idx);
        let result = self.idx;

        self.children
            .get_mut(&parent_id)
            .unwrap()
            .insert(child_idx, self.idx);
        self.idx += 1;

        result
    }

    /**
     * Generates a new octree with the given list
     * of stars and parameters. The list must be
     * sorted by magnitude beforehand
     **/
    pub fn generate(&mut self, list: &Vec<Particle>) {
        let root = self.start_generation(list);

        self.add_node(root);

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
                cat_idx += 1;

                let x = star.x;
                let y = star.y;
                let z = star.z;
                let mut added_num: i32;

                let root = self.nodes.get(self.root).expect("Error: root not found");
                let node_id = Octree::octant_id(x, y, z, level, &root.min, &root.max);

                if !self.id_map.contains_key(&node_id) {
                    // Octree node does not exist yet, create it
                    let octant = self.create_octant(node_id, x, y, z, level as i32);
                }
            }

            if (cat_idx >= list.len()) {
                // All stars added -> FINISHED
                break;
            }
        }
    }

    /**
     * Creates a new octant with the given parameters
     **/
    pub fn create_octant(&mut self, id: i64, x: f64, y: f64, z: f64, level: i32) -> i64 {
        let mut min: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let mut current = self.nodes.get(self.root).expect("Error: root not found");
        for l in 1..level {
            let hs: f64 = current.size.x / 2.0;
            let idx;

            if x <= current.min.x + hs {
                if y <= current.min.y + hs {
                    if z <= current.min.z + hs {
                        idx = 0;
                        min.set_from(&current.min);
                    } else {
                        idx = 1;
                        min.set(current.min.x, current.min.y, current.min.z + hs);
                    }
                } else {
                    if z <= current.min.z + hs {
                        idx = 2;
                        min.set(current.min.x, current.min.y + hs, current.min.z);
                    } else {
                        idx = 3;
                        min.set(current.min.x, current.min.y + hs, current.min.z + hs);
                    }
                }
            } else {
                if y <= current.min.y + hs {
                    if z <= current.min.z + hs {
                        idx = 4;
                        min.set(current.min.x + hs, current.min.y, current.min.z);
                    } else {
                        idx = 5;
                        min.set(current.min.x + hs, current.min.y, current.min.z + hs);
                    }
                } else {
                    if z <= current.min.z + hs {
                        idx = 6;
                        min.set(current.min.x + hs, current.min.y + hs, current.min.z);
                    } else {
                        idx = 7;
                        min.set(current.min.x + hs, current.min.y + hs, current.min.z + hs);
                    }
                }
            }

            //let is_none = self.children.get(&current.id).unwrap().get(idx).is_none();
            //if is_none {
            //    // Create kid
            //    let nhs: f64 = hs / 2.0;

            //    let x = min.x + nhs;
            //    let y = min.y + nhs;
            //    let z = min.z + nhs;
            //    let hs = nhs;

            //    let node = OctreeNode::new(x, y, z, hs, l);
            //    let curr_id = current.id;
            //    self.add_node_as_child(node, curr_id, idx);
            //}
            //let curr_id = current.id;
            //current = self
            //    .nodes
            //    .get_mut(self.children.get(&curr_id).unwrap().get(idx).unwrap())
            //    .expect("Error getting node");
        }

        current.id
    }

    /**
     * Computes the maximum axis-aligned bounding box
     * containing all the particles in the list
     **/
    fn start_generation(&self, list: &Vec<Particle>) -> OctreeNode {
        println!("Starting generation of octree");

        let mut max_dist = -1.0;
        let mut furthest = list.get(0).unwrap();

        for particle in list {
            let dist = (particle.x.powi(2) + particle.y.powi(2) + particle.z.powi(2)).sqrt();
            if dist > max_dist {
                max_dist = dist;
                furthest = &particle;
            }
        }

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
            let vol = Octree::compute_volume(&pos1, &pos0);
            if vol > volume {
                volume = vol;
                min = pos1.copy();
                max = pos0.copy();
            }
        }
        let half_size = (max.x - min.x) / 2.0;

        OctreeNode {
            id: 0,
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
            depth: 0,
            n_obj: 0,
            n_obj_own: 0,
            n_children: 0,
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

    /**
     * Gets the ID of the node which corresponds to the given
     * [x,y,z] position and level. The two least significant digits indicate
     * the level, and the rest of the digits indicate the child index in the level
     **/
    fn octant_id(x: f64, y: f64, z: f64, level: usize, rmin: &Vec3, rmax: &Vec3) -> i64 {
        if level == 0 {
            return 0;
        }

        let mut min: Vec3 = rmin.copy();
        let mut max: Vec3 = rmin.copy();

        let mut hs = (max.x - min.x) / 2.0;
        let mut hashv: [i32; 25] = [0; 25];
        hashv[0] = level as i32;

        for l in 1..level {
            if x <= min.x + hs {
                if y <= min.y + hs {
                    if z <= min.z + hs {
                        // Min stays the same!
                        hashv[l] = 0;
                    } else {
                        min.set(min.x, min.y, min.z + hs);
                        hashv[l] = 1;
                    }
                } else {
                    if z <= min.z + hs {
                        min.set(min.x, min.y + hs, min.z);
                        hashv[l] = 2;
                    } else {
                        min.set(min.x, min.y + hs, min.z + hs);
                        hashv[l] = 3;
                    }
                }
            } else {
                if y <= min.y + hs {
                    if z <= min.z + hs {
                        min.set(min.x + hs, min.y, min.z);
                        hashv[l] = 4;
                    } else {
                        min.set(min.x + hs, min.y, min.z + hs);
                        hashv[l] = 5;
                    }
                } else {
                    if z <= min.z + hs {
                        min.set(min.x + hs, min.y + hs, min.z);
                        hashv[l] = 6;
                    } else {
                        min.set(min.x + hs, min.y + hs, min.z + hs);
                        hashv[l] = 7;
                    }
                }
            }
            // Max is always half side away from min
            max.set(min.x + hs, min.y + hs, min.z + hs);
            hs = hs / 2.0;
        }

        return Octree::array_hash_code(hashv);
    }

    fn array_hash_code(a: [i32; 25]) -> i64 {
        let mut result: i64 = 1;
        let len: usize = a.len();

        for i in 0..len {
            result = 31 * result + a[i] as i64;
        }

        return result;
    }
}

/**
 * Defines an octree, a tree in which
 * each node is a cube and parts the
 * space into 8 children sub-cubes
 **/
pub struct OctreeNode {
    pub id: i64,
    pub min: Vec3,
    pub max: Vec3,
    pub centre: Vec3,
    pub size: Vec3,
    pub depth: i32,
    pub n_obj: i32,
    pub n_obj_own: i32,
    pub n_children: i32,
}

const MAX_DIST_CAP: f64 = 1.0e6;

impl OctreeNode {
    /**
     * Creates a shallow octree node with the given centre, half size and depth
     **/
    pub fn new(x: f64, y: f64, z: f64, half_size: f64, depth: i32) -> Self {
        OctreeNode {
            id: 0,
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
            centre: Vec3 { x: x, y: y, z: z },
            size: Vec3 {
                x: half_size * 2.0,
                y: half_size * 2.0,
                z: half_size * 2.0,
            },
            depth: depth,
            n_obj: 0,
            n_obj_own: 0,
            n_children: 0,
        }
    }
}
