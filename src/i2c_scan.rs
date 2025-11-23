// I2C scanner for nRF52
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use nrf52_rust_primer::hal::{bind_interrupts, peripherals, twim::{self, Twim}};
use embassy_time::Timer;
use nrf52_rust_primer::{self as _, info, led::Led};
use embassy_hal_internal::Peri;

bind_interrupts!(struct Irqs {TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;});

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

// Declare async tasks
#[embassy_executor::task]
async fn blink(pin: Peri<'static, crate::peripherals::P0_13>) {

    // Set up LED for visual feedback
    // let pin = p.P0_13;
    let mut led = Led::new(pin);

    loop {
        // Timekeeping is globally available, no need to mess with hardware timers.
        led.blink(2000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = nrf52_rust_primer::hal::init(Default::default());
    
    // Spawn LED blink task (runs concurrently in background)
    info!("Blinky Starting...");
    spawner.spawn(blink(p.P0_13)).unwrap();

    // Set up I2C (TWIM - Two Wire Interface Master)
    // Configure pins for I2C: P0_26 as SCL, P0_27 as SDA
    // Adjust these pins to match your hardware setup
    let config = twim::Config::default();
    let mut tx_buf = [0u8; 32];
    let mut i2c = Twim::new(p.TWISPI0, Irqs, p.P0_27, p.P0_26, config, &mut tx_buf);

    info!("Starting I2C address scan...");
    info!("Scanning addresses 0x08 to 0x77...");

    loop {
        
        // Scan through valid I2C addresses (0x08 to 0x77)
        // 0x00-0x07 and 0x78-0x7F are reserved
        for addr in 0x08..=0x77u8 {
            // Try to write to the address (with empty data)            
            match i2c.write(addr, &[0u8; 1]).await {
                Ok(()) => info!("{}Found device at address 0x{:02X}{}", GREEN, addr, RESET),
                Err(_) => info!("{}No device at address 0x{:02X}{}", RED, addr, RESET),
            }
            
            // Small delay between probes to avoid overwhelming the bus
            Timer::after_millis(2).await;
        }
        
        info!("========");
        
        // Wait before next scan
        Timer::after_secs(3).await;
    }
}