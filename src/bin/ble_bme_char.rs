#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_futures::join::join;

use nrf52_rust_primer::ble::nrf_ble::BLEWrapper;
use nrf52_rust_primer::ble::ble_services::{self, *};
use nrf52_rust_primer::peripherals::sensors::sensor_updates::{self, bme_update};
use nrf52_rust_primer::state::{TEMP_VAL, PRESSURE_VAL};

use nrf52_rust_primer::d_info;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    d_info!("Main script starting!");

    // Very finicky - HAL interrupts have to be given lower priority than softdeivce
    // this block needs to come before SoftDevice is enabled
    let p = sensor_updates::start_peripherals();

    // Starts softdevice and GATT server - needs to happen before mutex is initialized
    let (ble, server) = BLEWrapper::start_with_gatt::<BLEServer>(spawner, None, None, None, |sd| BLEServer::new(sd).unwrap()).await;

    // Return and print BLE address
    ble.get_ble_address().unwrap();

    // Initialize I2C Bus
    let i2c_mutex_wrapper = sensor_updates::start_i2c(p.P0_26, p.P0_27, p.TWISPI0);

    // Spawn bme680 task (runs concurrently in background)
    d_info!("BME680 Read starting...");
    let bme_delay_ms: u64 = 500;    // Frequency at which to read the sensor
    let bme_update_ms: u64 = 1000;  // Frequency at which to update the characteristic
    spawner.spawn(bme_update(i2c_mutex_wrapper, bme_delay_ms)).unwrap();

    // This loop will iterate every time either the update_fur or gatt_fur runs (so only upon disconnect)
    loop {

        // Advertise + wait for connection
        let conn = ble.advertise(true).await.unwrap();

        // Code for updating service characteristic
        // This joins multiple futures into 1
        let update_characteristics = join(
            ble_services::update_temperature(&server, &TEMP_VAL, bme_update_ms),
            ble_services::update_pressure(&server, &PRESSURE_VAL, bme_update_ms),
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