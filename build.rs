use std::fs;

fn main() {
    // Required for Cortex-M
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    // Decide which memory layout to use
    let memory_file = if cfg!(feature = "ble_memory") {
        "INCLUDE memory/memory_ble.x\n"
    } else if cfg!(feature = "default_memory") {
        "INCLUDE memory/memory_default.x\n"
    } else {
        panic!("One of 'default_memory' or 'ble_memory' must be enabled");
    };

    // 🔑 This creates memory.x for link.x to include
    fs::write("memory.x", memory_file).unwrap();

    // Ensure rebuilds when memory layouts change
    println!("cargo:rerun-if-changed=memory/memory_default.x");
    println!("cargo:rerun-if-changed=memory/memory_ble.x");
    println!("cargo:rerun-if-changed=Cargo.toml");
}