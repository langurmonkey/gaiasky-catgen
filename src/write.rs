use crate::data;
use crate::lod;

use std::io::Write;

use data::Particle;
use lod::Octant;
use lod::Octree;

use std::fs::OpenOptions;

pub fn write_metadata(octree: &Octree, output_dir: &str) {
    println!(
        "Writing metadata ({} nodes): {}/metadata.bin",
        octree.nodes.borrow().len(),
        output_dir
    );

    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("{}/{}", output_dir, "metadata.bin"))
        .expect("Error: file not found");

    // Number of nodes
    f.write_all(&octree.nodes.borrow().len().to_be_bytes());

    for node in octree.nodes.borrow().iter() {
        f.write_all(&node.id.0.to_be_bytes());
        f.write_all(&(node.centre.x as f32).to_be_bytes());
        f.write_all(&(node.centre.y as f32).to_be_bytes());
        f.write_all(&(node.centre.z as f32).to_be_bytes());
        f.write_all(&(node.size.x as f32).to_be_bytes());
        f.write_all(&(node.size.y as f32).to_be_bytes());
        f.write_all(&(node.size.z as f32).to_be_bytes());
        for i in 0..8_usize {
            // Children
            if node.has_child(i) {
                f.write_all(&(node.get_child(i).unwrap().0 as i32).to_be_bytes());
            } else {
                f.write_all(&(-1 as i32).to_be_bytes());
            }
        }
        f.write_all(&(node.level as i32).to_be_bytes());
        f.write_all(&(node.num_objects_rec.get()).to_be_bytes());
        f.write_all(&(node.num_objects.get()).to_be_bytes());
        f.write_all(&(node.num_children.get()).to_be_bytes());
    }
}

pub fn write_particles(octree: &Octree, list: Vec<Particle>, output_dir: &str) {
    let mut file_num = 0;
    for node in octree.nodes.borrow().iter() {
        let id_str = format!("{:08}", node.id.0);
        println!(
            "{}: Writing {} particles of node {} to {}/{}.bin",
            file_num,
            node.num_objects.get(),
            node.id.0,
            output_dir,
            id_str
        );
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(format!("{}/{}.bin", output_dir, id_str))
            .expect("Error: file not found");

        // Version marker
        f.write_all(&(-1_i32).to_be_bytes());
        // Version = 2
        f.write_all(&(2_i32).to_be_bytes());

        // Particles
        for star_idx in node.objects.borrow().iter() {
            let sb = list
                .get(*star_idx)
                .expect(&format!("Star not found: {}", *star_idx));

            // 64-bit floats
            f.write_all(&(sb.x).to_be_bytes());
            f.write_all(&(sb.y).to_be_bytes());
            f.write_all(&(sb.z).to_be_bytes());

            // 32-bit floats
            f.write_all(&(sb.pmx).to_be_bytes());
            f.write_all(&(sb.pmy).to_be_bytes());
            f.write_all(&(sb.pmz).to_be_bytes());
            f.write_all(&(sb.mualpha).to_be_bytes());
            f.write_all(&(sb.mudelta).to_be_bytes());
            f.write_all(&(sb.radvel).to_be_bytes());
            f.write_all(&(sb.appmag).to_be_bytes());
            f.write_all(&(sb.absmag).to_be_bytes());
            f.write_all(&(sb.col).to_be_bytes());
            f.write_all(&(sb.size).to_be_bytes());

            // 32-bit int
            f.write_all(&(sb.hip).to_be_bytes());

            // 64-bit int
            f.write_all(&(sb.id).to_be_bytes());

            // Names
            let mut names_concat = String::new();
            for name in sb.names.iter() {
                names_concat.push_str(name);
                names_concat.push_str("|");
            }
            names_concat.pop();

            // Names length
            f.write_all(&(names_concat.len() as i32).to_be_bytes());
            // Characters
            let mut buf: [u8; 2] = [0; 2];
            for ch in names_concat.chars() {
                ch.encode_utf8(&mut buf);
                f.write_all(&buf);
            }
        }
        file_num += 1;
    }
}
