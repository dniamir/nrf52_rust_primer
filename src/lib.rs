#![no_std]

use defmt_rtt as _;

// Re-export logging macros - change this one line to swap logging frameworks
pub use defmt::{debug, error, info, trace, warn};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    defmt::error!("Panic: {}", defmt::Debug2Format(info));
    cortex_m::peripheral::SCB::sys_reset()
}

#[defmt::panic_handler]
fn defmt_panic() -> ! {
    cortex_m::interrupt::disable();
    cortex_m::peripheral::SCB::sys_reset()
}

#[path = "lib/led.rs"]
pub mod led;