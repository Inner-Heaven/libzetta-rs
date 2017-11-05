fn main() {
    // Tell cargo to tell rustc to link the system libzfs_core
    // shared library.
    println!("cargo:rustc-link-lib=zfs_core");
}
