#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use embassy_nrf as _; // time driver
use nrf52_rust_primer as _;

use core::mem;

use defmt::{info, *};
use embassy_executor::Spawner;
use nrf_softdevice::ble::advertisement_builder::{Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList, ServiceUuid16,};
use nrf_softdevice::ble::peripheral;
use nrf_softdevice::ble;
use nrf_softdevice::{raw, Softdevice};

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);  // Enable Nordic BLE SoftDevice with specific config (more like on device settings)
    unwrap!(spawner.spawn(softdevice_task(sd)));            // Starts the soft device

    // Print SoftDevice BLE address - should already be a unique address from the FICR (Factory Information Configuration Registers)
    let [a, b, c, d, e, f] = ble::get_address(sd).bytes();
    defmt::info!("BLE addr: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", f, e, d, c, b, a);


    let mut config = peripheral::Config::default(); // Creates default config for peripheral (more like advertisement settings) (advertising interval, transmission power, channels, adress type, etc...)
    config.interval = 50;                                   // 50 BLE units of time (0.625 ms -> 31.25ms) - wakes up radio every 31ms to send packet for a few ms

    static ADV_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()     // Starts building the main BLE advertising payload
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])                                // Marks the device as discoverable and BLE only
        .services_16(ServiceList::Complete, &[ServiceUuid16::HEALTH_THERMOMETER])       // Advertises the device as a health thermometer
        .short_name("Hello")                                                            // Short device name to fit into limited advertisement space
        .build();

    // but we can put it in the scan data
    // so the full name is visible once connected
    // Defines extra data if requested
    static SCAN_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new().full_name("hello, Rust!").build();

    // Chooses a non-connectable but scanning advertisement mode - adds the settings previously chosen
    let adv = peripheral::NonconnectableAdvertisement::ScannableUndirected {
        adv_data: &ADV_DATA,
        scan_data: &SCAN_DATA,
    };

    // Starts BLE service
    unwrap!(peripheral::advertise(sd, adv, &config).await);
}