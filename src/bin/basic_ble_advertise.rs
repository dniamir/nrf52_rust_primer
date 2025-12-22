#![no_std]
#![no_main]

use nrf52_rust_primer::hal as _; // time driver
use embassy_executor::Spawner;

use nrf52_rust_primer::nrf_ble::BLEWrapper;
use nrf52_rust_primer as _;
use nrf52_rust_primer::d_info;  // Logging

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    // Start BLE subsystem
    let ble = BLEWrapper::start(spawner, None, None, None).await;

    // Return and print BLE address
    ble.get_ble_address().unwrap();

    // Start non-connectable advertising
    ble.advertise(false).await;

    d_info!("Main script complete");
}