#![no_std]

use defmt_rtt as _;

// HAL abstraction layer - conditionally compile based on feature flags
#[cfg(feature = "nrf")]
pub use embassy_nrf as embassy_hal;

#[cfg(feature = "stm32")]
pub use embassy_stm32 as embassy_hal;

// Generic HAL initialization function
pub fn init_hal(config: embassy_hal::config::Config) -> embassy_hal::Peripherals {
    embassy_hal::init(config)
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

// --- Base Modules (Top Level) ---
#[path = "lib/system/"]
pub mod system {
    pub mod state;
    pub mod ble_services;
    pub mod sensor_updates;
}

// --- BLE Module Group ---
#[path = "lib/d_ble/"]
pub mod d_ble {
    pub mod nrf_ble;
}

#[path = "lib/d_log/"]
pub mod d_log {
    pub mod dlogger;
}

#[path = "lib/d_peripherals/"]
pub mod d_peripherals {
    pub mod led;
    pub mod chip;
    pub mod chip_implementations;
    pub mod chip_map;

    #[path = "sensors/"]
    pub mod sensors {

        #[path = "bme680/bme680.rs"]
        pub mod bme680;
    }
}