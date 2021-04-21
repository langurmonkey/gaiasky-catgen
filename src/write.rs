use crate::data;
use crate::lod;

use memmap::MmapMut;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use data::Particle;
use lod::Octree;

pub fn write_metadata(octree: &Octree, output_dir: &str) {
    // Compute nubmer of undeleted nodes
    let mut num_nodes: i32 = 0;
    for node in octree.nodes.borrow().iter() {
        num_nodes = if node.deleted.get() {
            num_nodes
        } else {
            num_nodes + 1
        };
    }

    log::info!(
        ":: Writing metadata ({} nodes) to {}/metadata.bin",
        num_nodes,
        output_dir
    );

    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("{}/{}", output_dir, "metadata.bin"))
        .expect("Error: file not found");

    // Token to identify version annotation
    f.write_all(&(-1 as i32).to_be_bytes())
        .expect("Error writing");
    // Version 1
    f.write_all(&(1 as i32).to_be_bytes())
        .expect("Error writing");

    // Number of nodes
    f.write_all(&(num_nodes).to_be_bytes())
        .expect("Error writing");

    let mut written_nodes: i32 = 0;
    for node in octree.nodes.borrow().iter() {
        if node.deleted.get() {
            // Skip deleted
            continue;
        }
        f.write_all(&(node.id.0 as i64).to_be_bytes())
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

        written_nodes += 1;
    }

    log::info!(
        ":: Written {}:{} nodes (count:written) to metadata file",
        num_nodes,
        written_nodes
    )
}

#[allow(dead_code)]
pub fn write_particles(octree: &Octree, list: Vec<Particle>, output_dir: &str) {
    let mut file_num = 0;
    for node in octree.nodes.borrow().iter() {
        if node.deleted.get() {
            // Skip deleted
            continue;
        }
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
    log::info!("Written {} particle files", file_num);
}

#[allow(dead_code)]
pub fn write_particles_mmap(octree: &Octree, list: Vec<Particle>, output_dir: &str) {
    let mut file_num = 0;
    for node in octree.nodes.borrow().iter() {
        if node.deleted.get() {
            // Skip deleted
            continue;
        }

        // COMPUTE FILE SIZE
        // header 3 * i32
        let mut size = 32 * 3;
        // particles
        for star_idx in node.objects.borrow().iter() {
            if list.len() > *star_idx {
                // 3 * f64
                size += 8 * 3;
                // 10 * f32
                size += 4 * 10;
                // 1 * i32 hip
                size += 4 * 1;
                // 1 * i64 source_id
                size += 8 * 1;
                // 1 * i32 name_len
                size += 4 * 1;

                let sb = list
                    .get(*star_idx)
                    .expect(&format!("Star not found: {}", *star_idx));
                let mut name_size = 0;
                for name in sb.names.iter() {
                    name_size += name.len() + 1;
                }
                if name_size > 0 {
                    name_size -= 1;
                }
                // 1 * u16 * name_len
                size += 2 * name_size;
            }
        }

        // File name
        let id_str = format!("particles_{:06}", node.id.0);
        let particles_dir = format!("{}/particles", output_dir);
        std::fs::create_dir_all(Path::new(&particles_dir))
            .expect(&format!("Error creating directory: {}", particles_dir));
        let file_path = format!("{}/{}.bin", particles_dir, id_str);
        log::info!(
            "{}: Writing {} particles of node {} to {} ({} bytes)",
            file_num,
            node.num_objects.get(),
            node.id.0,
            file_path,
            size
        );

        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(file_path)
            .expect("Error opening memory mapped file");

        f.set_len(size as u64)
            .expect("Error setting size to memory mapped file");
        let mut mmap = unsafe { MmapMut::map_mut(&f).expect("Error creating memory map") };

        let mut i: usize = 0;
        // Version marker
        (&mut mmap[i..i + 4])
            .write_all(&(-1_i32).to_be_bytes())
            .expect("Error writing");
        i += 4;

        // Version = 2
        (&mut mmap[i..i + 4])
            .write_all(&(2_i32).to_be_bytes())
            .expect("Error writing");
        i += 4;

        // Size
        (&mut mmap[i..i + 4])
            .write_all(&(node.objects.borrow().len() as i32).to_be_bytes())
            .expect("Error writing");
        i += 4;

        // Particles
        for star_idx in node.objects.borrow().iter() {
            if list.len() > *star_idx {
                let sb = list
                    .get(*star_idx)
                    .expect(&format!("Star not found: {}", *star_idx));

                // 64-bit floats
                (&mut mmap[i..i + 8])
                    .write_all(&(sb.x).to_be_bytes())
                    .expect("Error writing");
                i += 8;
                (&mut mmap[i..i + 8])
                    .write_all(&(sb.y).to_be_bytes())
                    .expect("Error writing");
                i += 8;
                (&mut mmap[i..i + 8])
                    .write_all(&(sb.z).to_be_bytes())
                    .expect("Error writing");
                i += 8;

                // 32-bit floats
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.pmx).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.pmy).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.pmz).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.mualpha).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.mudelta).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.radvel).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.appmag).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.absmag).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.col).to_be_bytes())
                    .expect("Error writing");
                i += 4;
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.size).to_be_bytes())
                    .expect("Error writing");
                i += 4;

                // 32-bit int
                (&mut mmap[i..i + 4])
                    .write_all(&(sb.hip).to_be_bytes())
                    .expect("Error writing");
                i += 4;

                // 64-bit int
                (&mut mmap[i..i + 8])
                    .write_all(&(sb.id).to_be_bytes())
                    .expect("Error writing");
                i += 8;

                // Names
                let mut names_concat = String::new();
                for name in sb.names.iter() {
                    names_concat.push_str(name);
                    names_concat.push_str("|");
                }
                names_concat.pop();

                // Names length
                (&mut mmap[i..i + 4])
                    .write_all(&(names_concat.len() as i32).to_be_bytes())
                    .expect("Error writing");
                i += 4;

                // Characters
                let mut buf: [u16; 1] = [0; 1];
                for ch in names_concat.chars() {
                    ch.encode_utf16(&mut buf);
                    (&mut mmap[i..i + 2])
                        .write_all(&(buf[0]).to_be_bytes())
                        .expect("Error writing");
                    i += 2;
                }
            } else {
                log::error!(
                    "The needed star index is out of bounds: len:{}, idx:{}",
                    list.len(),
                    *star_idx
                );
            }
        }
        mmap.flush().expect("Error flushing memory map");
        file_num += 1;
    }
    log::info!("Written {} particle files", file_num);
}
