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
use nrf52_rust_primer::{self as _, led::Led, chip::Chip, chip::I2CMutexWrapper};
use nrf52_rust_primer::d_info;  // Logging

bind_interrupts!(struct Irqs {TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;});

// Static I2C bus protected by a Mutex for sharing between tasks
static I2C_MUTEX: StaticCell<Mutex<ThreadModeRawMutex, Twim<'static>>> = StaticCell::new();
static TX_BUF: StaticCell<[u8; 32]> = StaticCell::new();

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

// Async chip read
#[embassy_executor::task]
async fn chip_read(i2c_bus: I2CMutexWrapper) {

    // Do some simple chip reads
    d_info!("Setting up chip");

    let bme_address = 0x76;
    let chip = Chip::new_generic(i2c_bus, bme_address);

    loop {


        // Read register with generic register read
        let _field_val2 = chip.read_reg(0xD0).await.unwrap();

        d_info!("========");

        chip.write_reg(0x74, 0b11100011).await.unwrap();
        chip.read_reg(0x74).await.unwrap();

        d_info!("========");

        chip.write_reg(0x74, 0b00011100).await.unwrap();
        chip.read_reg(0x74).await.unwrap();

        d_info!("========");

        let reg_vals = &mut [0u8; 4];
        chip.read_regs(0x74, reg_vals).await.unwrap();

        d_info!("========");

        // Wait before next scan
        Timer::after_secs(3).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: nrf52_rust_primer::hal::Peripherals = nrf52_rust_primer::hal::init(Default::default());
    
    // Initialize I2C bus
    let config = twim::Config::default();
    let tx_buf = TX_BUF.init([0u8; 32]);
    let i2c_bus = Twim::new(p.TWISPI0, Irqs, p.P0_27, p.P0_26, config, tx_buf);
    let i2c_mutex = I2C_MUTEX.init(Mutex::new(i2c_bus));
    let i2c_mutex_wrapper = I2CMutexWrapper(i2c_mutex);

    // Spawn LED blink task (runs concurrently in background)
    d_info!("Blinky Starting...");
    spawner.spawn(blink(p.P0_13)).unwrap();
    
    // Spawn chip read task (runs concurrently in background)
    d_info!("Chip read Starting...");
    spawner.spawn(chip_read(i2c_mutex_wrapper)).unwrap();

    let mut count = 0;

    loop {
        count += 1;
        d_info!("Count: {}", count);
        Timer::after_secs(1).await;
    }
}