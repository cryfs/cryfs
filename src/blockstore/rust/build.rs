fn main() {
    let _build = cxx_build::bridges(vec!["src/blockstore/cppbridge.rs"].into_iter());

    println!("cargo:rerun-if-changed=src/blockstore/cppbridge.rs");
}
