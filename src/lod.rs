use crate::constants;
use crate::data;
use crate::parse;

use data::{BoundingBox, Particle, Vec3};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;

/**
 * An octree contains a root node,
 * an index and an array of nodes.
 **/
pub struct Octree {
    pub max_part: usize,
    pub postprocess: bool,
    pub child_count: usize,
    pub parent_count: usize,
    pub distpc_cap: f64,

    // From octantId to index in nodes vector
    pub nodes_idx: RefCell<HashMap<i64, usize>>,
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

            nodes_idx: RefCell::new(HashMap::new()),
            nodes: RefCell::new(Vec::new()),
            root: None,
        }
    }

    /**
     * Generates a the octree with the given list
     * of stars. The list must be
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
        let cat_size = list.len();
        for level in 0..=18 {
            log::info!(
                "Generating level {} ({} stars left)",
                level,
                cat_size - cat_idx
            );
            while cat_idx < cat_size {
                // Add stars to nodes until we reach max_part
                let star = list.get(cat_idx).expect("Error getting star");

                // Check if star is too far
                let dist = (star.x * star.x + star.y * star.y + star.z * star.z).sqrt();
                if dist * constants::U_TO_PC > self.distpc_cap {
                    log::info!(
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

                let octant_id = self.position_octant_id(x, y, z, level);
                let octant_i: usize;

                if !self.has_node(octant_id) {
                    // Octree node does not exist yet, create it
                    let (oc_i, n_cre) = self.create_octant(x, y, z, level);
                    octree_node_num += n_cre;

                    octant_i = oc_i;
                } else {
                    octant_i = self.get_node(octant_id);
                }

                // Add star to octant
                let added_num = self.nodes.borrow().get(octant_i).unwrap().add_obj(cat_idx);

                // Update max depth
                if level > depth {
                    depth = level;
                }
                octree_star_num += 1;
                cat_idx += 1;

                // Print every 10 M to stdout
                if cat_idx % 10_000_000 == 0 {
                    let pct = 100.0 * cat_idx as f64 / cat_size as f64;
                    let n_fill = (pct / 5.0) as usize;
                    let progress = String::from_utf8(vec![35; n_fill]).unwrap();
                    let bg = String::from_utf8(vec![45; 20 - n_fill]).unwrap();

                    println!(
                        "{}{}   {:.3}% ({}/{})",
                        progress, bg, pct, cat_idx, cat_size
                    );
                }

                if added_num >= self.max_part {
                    // Next level
                    break;
                }
            }

            if cat_idx >= cat_size {
                // All stars added -> FINISHED
                break;
            }
        }
        if depth == 18 && cat_idx < cat_size {
            log::info!(
                "WARN: Maximum depth reached ({}) and there are still {} stars left!",
                depth,
                cat_size - cat_idx
            );
        }
        log::info!(
            "GENERATION (1st round): {} nodes, {} stars",
            octree_node_num,
            octree_star_num
        );
        // Actually count nodes and stars to see if they match
        let (ncomputedstars, ncomputednodes) = self.nodes.borrow()[0].compute_numbers(&self);
        log::info!(
            "COMPUTED NUMBERS: {} nodes, {} stars",
            ncomputednodes,
            ncomputedstars
        );

        // Remove empty nodes by floating up objects
        let mut merged_nodes: usize = 0;
        let mut merged_objects: usize = 0;
        let nodes = self.nodes.borrow();
        // From deepest to root
        for level in (0..=depth).rev() {
            for node in nodes.iter() {
                if !node.deleted.get()
                    && !node.has_kids()
                    && node.level == level
                    && node.parent.is_some()
                {
                    let parent_id = node.parent.unwrap();
                    let parent_i = *self.nodes_idx.borrow().get(&parent_id.0).unwrap();
                    let parent = nodes.get(parent_i).unwrap();
                    let node_objects_count = node.objects.borrow().len();
                    let parent_objects_count = parent.objects.borrow().len();
                    if parent_objects_count == 0 {
                        // Add all node objects to parent objects, delete node

                        // Add objects from node to parent
                        let mut p_objects = parent.objects.borrow_mut();
                        for id in node.objects.borrow().iter() {
                            p_objects.push(*id);
                        }

                        // Remove objects from node
                        node.objects.borrow_mut().clear();

                        // Delete node from parent
                        let mut p_nodes = parent.children.borrow_mut();
                        for i in 0..8 {
                            if p_nodes[i].is_some() && p_nodes[i].unwrap().0 == node.id.0 {
                                p_nodes[i] = None;
                            }
                        }

                        // Mark deleted
                        node.deleted.set(true);

                        merged_objects += node_objects_count;
                        merged_nodes += 1;
                    }
                }
            }
        }
        log::info!(
            "Removed {} nodes due to being empty, {} objects floated",
            merged_nodes,
            merged_objects
        );
        octree_node_num -= merged_nodes;
        log::info!("GENERATION (2st round): {} nodes", octree_node_num);

        // User-defined post-process
        if self.postprocess {
            log::info!(
                "Post-processing octree: child_count={}, parent_count={}",
                self.child_count,
                self.parent_count
            );
            let mut merged_nodes_pp: usize = 0;
            let mut merged_objects_pp: usize = 0;
            let nodes = self.nodes.borrow();

            // From deepest to root
            for level in (0..=depth).rev() {
                for node in nodes.iter() {
                    if !node.deleted.get()
                        && !node.has_kids()
                        && node.level == level
                        && node.parent.is_some()
                    {
                        let parent_id = node.parent.unwrap();
                        let parent_i = *self.nodes_idx.borrow().get(&parent_id.0).unwrap();
                        let parent = nodes.get(parent_i).unwrap();
                        let node_objects_count = node.objects.borrow().len();
                        let parent_objects_count = parent.objects.borrow().len();
                        if node_objects_count <= self.child_count
                            && parent_objects_count <= self.parent_count
                        {
                            // Add all node objects to parent objects, delete node

                            // Add objects from node to parent
                            let mut p_objects = parent.objects.borrow_mut();
                            for id in node.objects.borrow().iter() {
                                p_objects.push(*id);
                            }

                            // Remove objects from node
                            node.objects.borrow_mut().clear();

                            // Delete node from parent
                            let mut p_nodes = parent.children.borrow_mut();
                            for i in 0..8 {
                                if p_nodes[i].is_some() && p_nodes[i].unwrap().0 == node.id.0 {
                                    p_nodes[i] = None;
                                }
                            }

                            // Mark deleted
                            node.deleted.set(true);

                            merged_objects_pp += node_objects_count;
                            merged_nodes_pp += 1;
                        }
                    }
                }
            }

            log::info!("POSTPROCESS STATS:");
            log::info!("    Merged nodes:    {}", merged_nodes_pp);
            log::info!("    Merged objects:  {}", merged_objects_pp);

            octree_node_num -= merged_nodes_pp;
            log::info!("GENERATION (final round): {} nodes", octree_node_num);
        }

        // Compute numbers
        self.nodes.borrow()[0].compute_numbers(&self);
        (octree_node_num, octree_star_num, depth)
    }

    fn has_node(&self, octant_id: OctantId) -> bool {
        self.nodes_idx.borrow().contains_key(&octant_id.0)
    }

    fn get_node(&self, octant_id: OctantId) -> usize {
        *self.nodes_idx.borrow().get(&octant_id.0).unwrap()
    }

    fn add_new_node(
        &self,
        new_id: OctantId,
        x: f64,
        y: f64,
        z: f64,
        half_size: f64,
        level: u32,
        parent: Option<OctantId>,
    ) -> usize {
        let new_idx = self.nodes.borrow().len();
        let octant = Octant::from_params(new_id.0, x, y, z, half_size, level, parent);
        // Add to list
        self.nodes.borrow_mut().push(octant);
        // Add to index
        self.nodes_idx.borrow_mut().insert(new_id.0, new_idx);
        // Return index
        new_idx
    }

    pub fn position_octant_id(&self, x: f64, y: f64, z: f64, level: u32) -> OctantId {
        if level == 0 {
            return OctantId(0);
        }
        let mut min = self.nodes.borrow().get(0).unwrap().min.copy();
        let max = self.nodes.borrow().get(0).unwrap().max.copy();
        let mut hs = (max.x - min.x) / 2.0;
        let mut id: String = String::new();
        for _ in 1..=level {
            if x <= min.x + hs {
                if y <= min.y + hs {
                    if z <= min.z + hs {
                        id.push('1');
                        // min stays the same
                    } else {
                        min.set(min.x, min.y, min.z + hs);
                        id.push('2');
                    }
                } else {
                    if z <= min.z + hs {
                        min.set(min.x, min.y + hs, min.z);
                        id.push('3');
                    } else {
                        min.set(min.x, min.y + hs, min.z + hs);
                        id.push('4');
                    }
                }
            } else {
                if y <= min.y + hs {
                    if z <= min.z + hs {
                        min.set(min.x + hs, min.y, min.z);
                        id.push('5');
                    } else {
                        min.set(min.x + hs, min.y, min.z + hs);
                        id.push('6');
                    }
                } else {
                    if z <= min.z + hs {
                        min.set(min.x + hs, min.y + hs, min.z);
                        id.push('7');
                    } else {
                        min.set(min.x + hs, min.y + hs, min.z + hs);
                        id.push('8');
                    }
                }
            }
            hs = hs / 2.0;
        }
        OctantId(parse::parse_i64(Some(&&id[..])))
    }

    /**
     * Creates a new octant with the given parameters
     **/
    pub fn create_octant(&self, x: f64, y: f64, z: f64, level: u32) -> (usize, usize) {
        let mut min: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let mut n_created: usize = 0;
        // start at root, which is always 0
        let mut current_i: usize = 0;
        let mut current = OctantId(0);
        for l in 1..=level {
            let hs: f64 = self.nodes.borrow().get(current_i).unwrap().size.x / 2.0;
            let idx;

            let cmin = self.nodes.borrow().get(current_i).unwrap().min;

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

            // If node does not exist in child list, create it
            if !self.nodes.borrow().get(current_i).unwrap().has_child(idx) {
                // Create kid
                let nhs: f64 = hs / 2.0;

                let x = min.x + nhs;
                let y = min.y + nhs;
                let z = min.z + nhs;

                let node_id = self.position_octant_id(x, y, z, l);
                if self.nodes_idx.borrow().contains_key(&node_id.0) {
                    panic!("Node {} already exists!!!", node_id.0);
                }
                self.add_new_node(node_id, x, y, z, nhs, l, Some(current));
                self.nodes
                    .borrow()
                    .get(current_i)
                    .unwrap()
                    .add_child(idx, node_id);
                n_created += 1;
            }
            current = self
                .nodes
                .borrow()
                .get(current_i)
                .unwrap()
                .get_child(idx)
                .expect("OctantId does not exist!");
            current_i = *self.nodes_idx.borrow().get(&current.0).unwrap();
        }

        (current_i, n_created)
    }

    /**
     * Computes the maximum axis-aligned bounding box
     * containing all the particles in the list and
     * sets up the root node of this octree.
     **/
    fn start_generation(&self, list: &Vec<Particle>) {
        log::info!("Starting generation of octree");

        let mut min = Vec3::with(1.0e50);
        let mut max = Vec3::with(-1.0e50);

        for particle in list {
            // Min
            if particle.x < min.x {
                min.x = particle.x;
            }
            if particle.y < min.y {
                min.y = particle.y;
            }
            if particle.z < min.z {
                min.z = particle.z;
            }

            // Max
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
        let size = f64::max(f64::max(bx.dim.z, bx.dim.y), bx.dim.x);
        let half_size = size / 2.0;

        let root = Octant {
            id: OctantId(0),
            min: Vec3::new(
                bx.cnt.x - half_size,
                bx.cnt.y - half_size,
                bx.cnt.z - half_size,
            ),
            max: Vec3::new(
                bx.cnt.x + half_size,
                bx.cnt.y + half_size,
                bx.cnt.z + half_size,
            ),
            centre: bx.cnt.copy(),
            size: Vec3 {
                x: size,
                y: size,
                z: size,
            },
            level: 0,

            num_objects: Cell::new(0),
            num_objects_rec: Cell::new(0),
            num_children: Cell::new(0),
            num_children_rec: Cell::new(0),

            deleted: Cell::new(false),

            parent: None,
            children: RefCell::new([None; 8]),
            objects: RefCell::new(Vec::new()),
        };

        // Volume of root node in pc^3
        let vol = f64::powi(size * constants::U_TO_PC, 3);
        log::info!(
            "Octree root node generated with min: {}, max: {}, centre: {}, volume: {} pc3",
            root.min,
            root.max,
            root.centre,
            vol
        );
        // Add to list
        self.nodes.borrow_mut().push(root);
        // Add root to index, id: 0, idx: 0
        self.nodes_idx.borrow_mut().insert(0, 0);
    }

    #[allow(dead_code)]
    pub fn count_nodes(&self) -> usize {
        let mut count: usize = 0;
        for node in self.nodes.borrow().iter() {
            if !node.deleted.get() {
                count += 1;
            }
        }
        count
    }

    pub fn print(&self) {
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
pub struct OctantId(pub i64);

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
    pub num_children_rec: Cell<i32>,

    pub deleted: Cell<bool>,

    pub parent: Option<OctantId>,
    pub children: RefCell<[Option<OctantId>; 8]>,
    pub objects: RefCell<Vec<usize>>,
}

impl Octant {
    /**
     * Creates a shallow octree node with the given centre, half size and depth
     **/
    pub fn from_params(
        id: i64,
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
            num_children_rec: Cell::new(0),

            deleted: Cell::new(false),

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

    #[allow(dead_code)]
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
                    Some(child) => {
                        let idx = *octree.nodes_idx.borrow().get(&child.0).unwrap();
                        octree.nodes.borrow().get(idx).unwrap().print(i, octree)
                    }
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
    pub fn compute_numbers(&self, octree: &Octree) -> (i32, i32) {
        let mut num_objects_rec = self.get_num_objects();
        self.num_objects.set(num_objects_rec);

        let mut num_children_rec = 0;
        for ch in 0..8 {
            if self.has_child(ch) {
                num_children_rec += 1;
            }
        }
        self.num_children.set(num_children_rec);

        // Recursively count objects
        for i in 0..8 {
            if self.children.borrow()[i].is_some() {
                let id = self.children.borrow()[i].unwrap();
                let idx = *octree.nodes_idx.borrow().get(&id.0).unwrap();
                let (objs, ch) = octree.nodes.borrow()[idx].compute_numbers(octree);

                num_objects_rec += objs;
                num_children_rec += ch;
            }
        }

        self.num_objects_rec.set(num_objects_rec);
        self.num_children_rec.set(num_children_rec);

        (num_objects_rec, num_children_rec)
    }

    pub fn get_num_objects(&self) -> i32 {
        self.objects.borrow().len() as i32
    }
}
