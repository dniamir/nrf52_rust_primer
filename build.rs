use std::fs;
use std::env;

fn main() {
    // --- 1. Configure Linker Arguments ---
    // These tell rustc how to link the final binary
    println!("cargo:rustc-link-arg-bins=--nmagic");
    // This is vital: link.x is likely the *base* script provided by cortex-m-rt
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    // --- 2. Define Feature-Dependent Logic ---
    let (feature_name, _memory_file_path, include_path_content) = if env::var("CARGO_FEATURE_BLE_MEMORY").is_ok() {
        ("ble_memory", "./memory/memory_ble.x", "memory/memory_ble.x")
    } else if env::var("CARGO_FEATURE_DEFAULT_MEMORY").is_ok() {
        ("default_memory", "./memory/memory_default.x", "memory/memory_default.x")
    } else {
        panic!("One of 'default_memory' or 'ble_memory' features must be enabled");
    };
    
    // --- 3. Ensure Cargo Invalidates Cache When Features Change ---
    // This explicitly tracks the environment variables Cargo sets for features.
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_BLE_MEMORY");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DEFAULT_MEMORY");
    // Ensure rebuilds when the source memory layouts change
    println!("cargo:rerun-if-changed=memory/memory_default.x");
    println!("cargo:rerun-if-changed=memory/memory_ble.x");


    // --- 4. Generate the Proxy memory.x file ---
    // The content is an INCLUDE directive pointing to the selected file
    let file_content = format!("INCLUDE {}\n", include_path_content);

    // Write the proxy file to the root directory
    fs::write("memory.x", file_content).expect("Failed to write memory.x");
    
    // Crucial Step: Tell Cargo to track *this specific output file* for timestamp checks
    // This ensures that when we switch features, even if the content happens to be identical 
    // to a previous run, Cargo knows the dependency chain changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("Selected memory feature: {}", feature_name);
}