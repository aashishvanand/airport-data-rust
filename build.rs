use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("airports.json.gz");

    let json_data = fs::read("data/airports.json").expect("Failed to read data/airports.json");

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&json_data).unwrap();
    let compressed = encoder.finish().unwrap();

    fs::write(&dest_path, &compressed).unwrap();

    println!("cargo:rerun-if-changed=data/airports.json");
    println!("cargo:rerun-if-changed=build.rs");
}
