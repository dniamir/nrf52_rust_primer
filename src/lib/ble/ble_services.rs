use core::sync::atomic::Ordering;
use embassy_time::Timer;
use core::sync::atomic::{AtomicI32, AtomicU32};

use nrf_softdevice::ble::gatt_server;
use crate::{dlogger::DLogger, d_info};  // Logging

/// GATT SERVICES (there are multiple)
/// For examples and library documentation
/// https://github.com/embassy-rs/nrf-softdevice/tree/master

// 16 bit UUIDs are predetermined by the BLE library
#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct BatteryService {
    #[characteristic(uuid = "2a19", read, notify)]
    pub battery_level: u8,
}

// 128 bit UUIDs are custom and globally unique
#[nrf_softdevice::gatt_service(uuid = "9e7312e0-2354-11eb-9f10-fbc30a62cf38")]
pub struct SensorService {

    #[characteristic(uuid = "9e7312e0-2354-11eb-9f10-fbc30a63cf41", read, notify)]
    #[descriptor(uuid="2901", value="temperature_c")]  // Doesn't seem to do anything
    pub temperature_c: i32,

    #[characteristic(uuid = "9e7312e0-2354-11eb-9f10-fbc30a63cf42", read, notify)]
    #[descriptor(uuid="2901", value="pressure_pa")]  // Doesn't seem to do anything
    pub pressure_pa: u32,
}

// GATT SERVER (there can only be one)

#[nrf_softdevice::gatt_server]
pub struct BLEServer {
    pub batt_service: BatteryService,
    pub sensor_service: SensorService,
}

// Create the gatt_future to run later
// gatt_server::run is an async function that returns a future
pub fn my_gatt_server<'a>(conn: &'a nrf_softdevice::ble::Connection, server: &'a BLEServer) -> impl core::future::Future<Output = ()> + 'a {
    async move {
        let _ = gatt_server::run(conn, server, handle_ble_event).await;
    }
}

// Define gatt server services
fn handle_ble_event(e: BLEServerEvent) {
    match e {
        // Battery service
        BLEServerEvent::BattService(e) => match e {
            BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
                d_info!("battery notifications: {}", notifications);
            }
        },

        // Sensor service
        BLEServerEvent::SensorService(e) => match e {
            SensorServiceEvent::TemperatureCCccdWrite { notifications } => {
                d_info!("temperature_c notifications: {}", notifications);
            }
            SensorServiceEvent::PressurePaCccdWrite { notifications } => {
                d_info!("pressure_c notifications: {}", notifications);
            }
        },
    }
}

pub async fn update_temperature(server: &BLEServer, atomic: &AtomicI32) {
    loop {
        Timer::after_millis(1000).await;

        let char_val = atomic.load(Ordering::Relaxed);

        let _ = server.sensor_service.temperature_c_set(&char_val);
        d_info!("Updated temperature_c characteristic: {}", char_val);
        DLogger::d_sep();
    }
}

pub async fn update_pressure(server: &BLEServer, atomic: &AtomicU32) {
    loop {
        Timer::after_millis(1000).await;

        let char_val = atomic.load(Ordering::Relaxed);

        let _ = server.sensor_service.pressure_pa_set(&char_val);
        d_info!("Updated pressure_pa characteristic: {}", char_val);
        DLogger::d_sep();
    }
}