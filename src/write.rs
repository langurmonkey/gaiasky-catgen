use crate::data;
use crate::lod;

use std::{io::Write, path::Path};

use data::Particle;
use lod::Octree;

use std::fs::OpenOptions;

pub fn write_metadata(octree: &Octree, output_dir: &str) {
    log::info!(
        ":: Writing metadata ({} nodes) to {}/metadata.bin",
        octree.nodes.borrow().len(),
        output_dir
    );

    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("{}/{}", output_dir, "metadata.bin"))
        .expect("Error: file not found");

    // Number of nodes
    f.write_all(&(octree.nodes.borrow().len() as i32).to_be_bytes())
        .expect("Error writing");

    for node in octree.nodes.borrow().iter() {
        f.write_all(&(node.id.0 as i32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.centre.x as f32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.centre.y as f32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.centre.z as f32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.size.x as f32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.size.y as f32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.size.z as f32).to_be_bytes())
            .expect("Error writing");
        for i in 0..8_usize {
            // Children
            if node.has_child(i) {
                f.write_all(&(node.get_child(i).unwrap().0 as i32).to_be_bytes())
                    .expect("Error writing");
            } else {
                f.write_all(&(-1 as i32).to_be_bytes())
                    .expect("Error writing");
            }
        }
        f.write_all(&(node.level as i32).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.num_objects_rec.get()).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.num_objects.get()).to_be_bytes())
            .expect("Error writing");
        f.write_all(&(node.num_children.get()).to_be_bytes())
            .expect("Error writing");
    }
}

pub fn write_particles(octree: &Octree, list: Vec<Particle>, output_dir: &str) {
    let mut file_num = 0;
    for node in octree.nodes.borrow().iter() {
        let id_str = format!("particles_{:06}", node.id.0);
        let particles_dir = format!("{}/particles", output_dir);
        std::fs::create_dir_all(Path::new(&particles_dir))
            .expect(&format!("Error creating directory: {}", particles_dir));
        let file_path = format!("{}/{}.bin", particles_dir, id_str);
        log::info!(
            "{}: Writing {} particles of node {} to {}",
            file_num,
            node.num_objects.get(),
            node.id.0,
            file_path
        );
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_path)
            .expect("Error: file not found");

        // Version marker
        f.write_all(&(-1_i32).to_be_bytes()).expect("Error writing");
        // Version = 2
        f.write_all(&(2_i32).to_be_bytes()).expect("Error writing");

        // Size
        f.write_all(&(node.objects.borrow().len() as i32).to_be_bytes())
            .expect("Error writing");

        // Particles
        for star_idx in node.objects.borrow().iter() {
            if list.len() > *star_idx {
                let sb = list
                    .get(*star_idx)
                    .expect(&format!("Star not found: {}", *star_idx));

                // 64-bit floats
                f.write_all(&(sb.x).to_be_bytes()).expect("Error writing");
                f.write_all(&(sb.y).to_be_bytes()).expect("Error writing");
                f.write_all(&(sb.z).to_be_bytes()).expect("Error writing");

                // 32-bit floats
                f.write_all(&(sb.pmx).to_be_bytes()).expect("Error writing");
                f.write_all(&(sb.pmy).to_be_bytes()).expect("Error writing");
                f.write_all(&(sb.pmz).to_be_bytes()).expect("Error writing");
                f.write_all(&(sb.mualpha).to_be_bytes())
                    .expect("Error writing");
                f.write_all(&(sb.mudelta).to_be_bytes())
                    .expect("Error writing");
                f.write_all(&(sb.radvel).to_be_bytes())
                    .expect("Error writing");
                f.write_all(&(sb.appmag).to_be_bytes())
                    .expect("Error writing");
                f.write_all(&(sb.absmag).to_be_bytes())
                    .expect("Error writing");
                f.write_all(&(sb.col).to_be_bytes()).expect("Error writing");
                f.write_all(&(sb.size).to_be_bytes())
                    .expect("Error writing");

                // 32-bit int
                f.write_all(&(sb.hip).to_be_bytes()).expect("Error writing");

                // 64-bit int
                f.write_all(&(sb.id).to_be_bytes()).expect("Error writing");

                // Names
                let mut names_concat = String::new();
                for name in sb.names.iter() {
                    names_concat.push_str(name);
                    names_concat.push_str("|");
                }
                names_concat.pop();

                // Names length
                f.write_all(&(names_concat.len() as i32).to_be_bytes())
                    .expect("Error writing");
                // Characters
                let mut buf: [u16; 1] = [0; 1];
                for ch in names_concat.chars() {
                    ch.encode_utf16(&mut buf);
                    f.write_all(&(buf[0]).to_be_bytes()).expect("Error writing");
                }
            } else {
                log::error!(
                    "The needed star index is out of bounds: len:{}, idx:{}",
                    list.len(),
                    *star_idx
                );
            }
        }
        file_num += 1;
    }
}
