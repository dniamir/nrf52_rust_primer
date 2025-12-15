use std::fs;

fn main() {
    // Required for Cortex-M
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    // Force rebuild when feature set changes
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_BLE_MEMORY");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DEFAULT_MEMORY");

    // Always regenerate memory.x
    let _ = fs::remove_file("memory.x");

    // Decide which memory layout to use
    let memory_file = if cfg!(feature = "ble_memory") {
        "INCLUDE memory/memory_ble.x\n"
    } else if cfg!(feature = "default_memory") {
        "INCLUDE memory/memory_default.x\n"
    } else {
        panic!("One of 'default_memory' or 'ble_memory' must be enabled");
    };

    // This creates memory.x for link.x to include
    fs::write("memory.x", memory_file).expect("Failed to write memory.x");

    // Ensure rebuilds when memory layouts change
    println!("cargo:rerun-if-changed=memory/memory_default.x");
    println!("cargo:rerun-if-changed=memory/memory_ble.x");
    println!("cargo:rerun-if-changed=Cargo.toml");
}