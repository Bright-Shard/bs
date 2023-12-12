fn main() {
    // Make rust compile the binary with our link script
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        root.join("link.ld").to_str().unwrap()
    );
}
