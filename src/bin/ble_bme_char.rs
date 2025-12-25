#![no_std]
#![no_main]

use static_cell::StaticCell;

use core::sync::atomic::Ordering;

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_futures::select::{select, Either};
use embassy_futures::join::join;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use nrf52_rust_primer::embassy_hal::{self, interrupt::Priority, bind_interrupts, peripherals, twim::{self, Twim}};
use nrf52_rust_primer::ble::nrf_ble::BLEWrapper;
use nrf52_rust_primer::ble::ble_services::{self, *};
use nrf52_rust_primer::peripherals::sensors::bme680::BME680;
use nrf52_rust_primer::peripherals::chip_implementations::I2CMutexWrapper;
use nrf52_rust_primer::state::{TEMP_VAL, PRESSURE_VAL};
use nrf52_rust_primer::d_info;

// Static I2C bus protected by a Mutex for sharing between tasks
pub type I2CMutex = &'static Mutex<ThreadModeRawMutex, Twim<'static>>;
bind_interrupts!(struct Irqs {TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;});
static I2C_MUTEX: StaticCell<Mutex<ThreadModeRawMutex, Twim<'static>>> = StaticCell::new();
static TX_BUF: StaticCell<[u8; 32]> = StaticCell::new();

// Async bme680 reads
#[embassy_executor::task]
async fn chip_read(i2c_bus: I2CMutexWrapper) {

    // Do some simple chip reads
    d_info!("Setting up BME680");

    let mut bme = BME680::new(i2c_bus, 0x76).await.unwrap();
    bme.config(1).await.expect("Unable to configure BME680");
    loop {

        // Read register with generic register read
        bme.chip.read_field("chip_id").await.unwrap();
        bme.chip.read_reg(0xD0).await.unwrap();

        let temp_val = bme.read_temperature().await.unwrap();
        let pressure_val = bme.read_pressure().await.unwrap();

        d_info!("========");

        // Send data to channel
        TEMP_VAL.store(temp_val, Ordering::Relaxed);
        PRESSURE_VAL.store(pressure_val, Ordering::Relaxed);

        // Wait before next scan
        Timer::after_secs(3).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    d_info!("Main script starting!");

    // Very finicky - HAL interrupts have to be given lower priority than softdeivce
    // this block needs to come before SoftDevice is enabled
    let mut ecfg = embassy_hal::config::Config::default();
    ecfg.gpiote_interrupt_priority = Priority::P2;
    ecfg.time_interrupt_priority   = Priority::P2; // for time-driver-rtc1
    let p = embassy_hal::init(ecfg);

    // Starts softdevice and GATT server - needs to happen before mutex is initialized
    let (ble, server) = BLEWrapper::start_with_gatt::<BLEServer>(spawner, None, None, None, |sd| BLEServer::new(sd).unwrap()).await;

    // Return and print BLE address
    ble.get_ble_address().unwrap();

    // Initialize I2C bus config
    let mut config = twim::Config::default();
    config.frequency = twim::Frequency::K100;

    // Initialize I2C bus
    let tx_buf = TX_BUF.init([0u8; 32]);
    let i2c_bus = Twim::new(p.TWISPI0, Irqs, p.P0_27, p.P0_26, config, tx_buf);
    let i2c_mutex = I2C_MUTEX.init(Mutex::new(i2c_bus));
    let i2c_mutex_wrapper = I2CMutexWrapper(i2c_mutex);

    // Spawn bme680 task (runs concurrently in background)
    d_info!("BME680 Read starting...");
    spawner.spawn(chip_read(i2c_mutex_wrapper)).unwrap();

    // This loop will iterate every time either the update_fur or gatt_fur runs (so only upon disconnect)
    loop {

        // Advertise + wait for connection
        let conn = ble.advertise(true).await.unwrap();

        // Code for updating service characteristic
        // This joins multiple futures into 1
        let update_characteristics = join(
            ble_services::update_temperature(&server, &TEMP_VAL),
            ble_services::update_pressure(&server, &PRESSURE_VAL),
        );
        
        // Run the GATT server on the connection. This returns when the connection gets disconnected.
        let gatt_server_fut = ble_services::my_gatt_server(&conn, &server);

        // These are both async functions
        match select(gatt_server_fut, update_characteristics).await {
            Either::First(e) => d_info!("Device disonnected: {:?}", e),     // If the first passed future finishes first
            Either::Second(_) => {},                                            // If the second passed future finished first (is an infite loop, should never finish)
        };
    }
}