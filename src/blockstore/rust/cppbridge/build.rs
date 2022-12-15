const BRIDGE_FILES: &[&str] = &[
    "src/blockstore.rs",
    "src/blobstore.rs",
    "src/fsblobstore.rs",
];

fn main() {
    let _build = cxx_build::bridges(BRIDGE_FILES.iter());

    for file in BRIDGE_FILES {
        println!("cargo:rerun-if-changed={}", file);
    }
}
