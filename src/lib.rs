#![no_std]

use defmt_rtt as _;

// HAL abstraction layer - conditionally compile based on feature flags
#[cfg(feature = "nrf")]
pub use embassy_nrf as hal;

#[cfg(feature = "stm32")]
pub use embassy_stm32 as hal;

// Generic HAL initialization function
pub fn init_hal(config: hal::config::Config) -> hal::Peripherals {
    hal::init(config)
}

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

#[path = "lib/chip.rs"]
pub mod chip;

#[path = "lib/chip_map.rs"]
pub mod chip_map;

#[path = "lib/dlogger.rs"]
pub mod dlogger;