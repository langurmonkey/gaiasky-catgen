mod data;
mod load;
mod util;

fn main() {
    let list = load::load_dir("data/*").expect("Error loading data");

    println!("{} particles loaded successfully", list.len());

    let n = std::cmp::min(50, list.len());
    for i in 0..n {
        let star = list.get(i).expect("Error: Star is weird");
        println!("{}: [{},{},{}]", i, star.x, star.y, star.z);
    }
}
