use std::env;

fn main() {
    let mut current_dir = env::current_dir().unwrap();

    current_dir.pop();
    current_dir.push("_dist");

    println!(
        "cargo:rustc-env=PLAYFAIR_DIST_DIR={}",
        current_dir.display()
    );
}
