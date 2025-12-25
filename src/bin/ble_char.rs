#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_futures::select::{select, Either};

use nrf52_rust_primer::embassy_hal::{self, interrupt::Priority}; // time driver
use nrf52_rust_primer::ble::nrf_ble::BLEWrapper;
use nrf52_rust_primer::ble::ble_services::{*, self};
use nrf52_rust_primer::d_info;  // Logging

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    d_info!("Main script starting!");

    // Very finicky - HAL interrupts have to be given lower priority than softdeivce
    // this block needs to come before softdevice is enabled
    let mut ecfg = embassy_hal::config::Config::default();
    ecfg.gpiote_interrupt_priority = Priority::P2;
    ecfg.time_interrupt_priority   = Priority::P2; // for time-driver-rtc1
    let _p = embassy_hal::init(ecfg);

    // Starts softdevice and GATT server
    let (ble, server) = BLEWrapper::start_with_gatt::<BLEServer>(spawner, None, None, None, |sd| BLEServer::new(sd).unwrap()).await;

    // Return and print BLE address
    ble.get_ble_address().unwrap();

    // This loop will iterate every time either the update_fur or gatt_fur runs (so only upon disconnect)
    loop {

        // Advertise + wait for connection
        let conn = ble.advertise(true).await.unwrap();

        // Code for updating service characteristic
        let mut count: i32 = 0;
        let update_fut = async {
            loop {
                Timer::after_millis(1000).await;
                count += 1;
                let _ = server.sensor_service.temperature_c_set(&count);
                d_info!("Updated characteristic with value: {}", count);
            }
        };
        
        // Run the GATT server on the connection. This returns when the connection gets disconnected.
        let gatt_server_fut = ble_services::my_gatt_server(&conn, &server);

        // These are both async functions
        match select(gatt_server_fut, update_fut).await {
            Either::First(e) => d_info!("Device disonnected: {:?}", e),     // If the first passed future finishes first
            Either::Second(_) => {},                                            // If the second passed future finished first (is an infite loop, should never finish)
        };
    }
}