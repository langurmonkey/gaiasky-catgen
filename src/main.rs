mod data;
mod load;
mod util;

fn main() {
    let list = load::load_dir("data/*").expect("Error loading data");

    println!("{} particles loaded successfully", list.len());
    //load::load_file("data/GaiaSource_000000-003111.csv.gz");
}
