#![no_std]
#![no_main]

use static_cell::StaticCell;

use embassy_nrf as _; // time driver
use embassy_executor::Spawner;

use nrf52_rust_primer::nrf_ble::BLEWrapper;
use nrf52_rust_primer as _;
use nrf52_rust_primer::d_info;  // Logging

static BLE_WRAPPER: StaticCell<BLEWrapper> = StaticCell::new();  // Have to use a static BLE wrapper so that it exists after main is complete

#[embassy_executor::task]
async fn softdevice_task(ble_wrapper: &'static BLEWrapper) -> ! {
    ble_wrapper.sd.unwrap().run().await  // This never completes, so this task runs forever
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    d_info!("Hello World!");

    // Create BLE wrapper class
    let ble_wrapper = BLE_WRAPPER.init(BLEWrapper::create_default().unwrap());
    ble_wrapper.set_default_sd_config().unwrap();
    ble_wrapper.enable().unwrap();

    // Log BLE address
    ble_wrapper.get_ble_address().unwrap();

    ble_wrapper.set_default_adv_config("Rust", "MyRust").unwrap();
    ble_wrapper.adv_config.interval = 50;  //  50 BLE units of time (0.625 ms -> 31.25ms) - wakes up radio every 31ms to send packet for a few ms

    // Start soft device
    spawner.spawn(softdevice_task(ble_wrapper)).unwrap();

    // Starts BLE service
    ble_wrapper.advertise().await.unwrap();
}