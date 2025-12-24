use core::sync::atomic::{AtomicI32, AtomicU32};

// Atomics for sharing data between threads
pub static TEMP_VAL: AtomicI32 = AtomicI32::new(0);
pub static PRESSURE_VAL: AtomicU32 = AtomicU32::new(0);
