// I2C scanner for nRF52
#![no_main]
#![no_std]

use static_cell::StaticCell;

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_hal_internal::Peri;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use nrf52_rust_primer::hal::{bind_interrupts, peripherals, twim::{self, Twim}};
use nrf52_rust_primer::{self as _, led::Led};
use nrf52_rust_primer::{dlogger::DLogger, d_info};

bind_interrupts!(struct Irqs {TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;});

// Static I2C bus protected by a Mutex for sharing between tasks
static I2C_BUS: StaticCell<Mutex<ThreadModeRawMutex, Twim<'static>>> = StaticCell::new();
static TX_BUF: StaticCell<[u8; 32]> = StaticCell::new();

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

// Declare async tasks
// Async Blinky
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

// Async i2c
#[embassy_executor::task]
async fn i2c_scan(i2c_bus: &'static Mutex<ThreadModeRawMutex, Twim<'static>>) {

    loop {
        // Scan through valid I2C addresses (0x08 to 0x77)
        // 0x00-0x07 and 0x78-0x7F are reserved
        d_info!("Starting I2C address scan...");
        d_info!("Scanning addresses 0x08 to 0x77...");
        for addr in 0x08..=0x77u8 {
            // Try to write to the address (with empty data)            
            let result = {
                let mut i2c = i2c_bus.lock().await;
                i2c.write(addr, &[0u8; 1]).await
            };
            match result {
                Ok(()) => d_info!("{}Found device at address 0x{:02X}{}", GREEN, addr, RESET),
                Err(_) => d_info!("{}No device at address 0x{:02X}{}", RED, addr, RESET),
            }
            
            // Small delay between probes to avoid overwhelming the bus
            Timer::after_millis(2).await;
        }

        // Wait before next scan
        DLogger::d_sep();
        Timer::after_secs(3).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: nrf52_rust_primer::hal::Peripherals = nrf52_rust_primer::hal::init(Default::default());
    
    // Initialize I2C bus
    let config = twim::Config::default();
    let tx_buf = TX_BUF.init([0u8; 32]);
    let i2c = Twim::new(p.TWISPI0, Irqs, p.P0_27, p.P0_26, config, tx_buf);
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    // Spawn LED blink task (runs concurrently in background)
    d_info!("Blinky Starting...");
    spawner.spawn(blink(p.P0_13)).unwrap();
    
    // Spawn i2c scan task (runs concurrently in background)
    d_info!("I2C Scan Starting...");
    spawner.spawn(i2c_scan(i2c_bus)).unwrap();

    let mut count = 0;

    loop {
        count += 1;
        d_info!("Count: {}", count);
        Timer::after_secs(1).await;
    }
}