fn main() {
    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("backend manifest dir"),
    );
    let elf_path = manifest_dir
        .join("../program/elf/world-id-root-replicator-program")
        .canonicalize()
        .expect("checked-in SP1 ELF");

    println!("cargo:rerun-if-changed={}", elf_path.display());
    println!(
        "cargo:rustc-env=SP1_ELF_world-id-root-replicator-program={}",
        elf_path.display()
    );
}
