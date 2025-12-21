use heapless::String;
use core::fmt::Write;
use core::mem;

use nrf_softdevice::{ble, raw, Softdevice};
use nrf_softdevice::ble::peripheral;
use nrf_softdevice::ble::peripheral::{ConnectableAdvertisement, NonconnectableAdvertisement};
use nrf_softdevice::ble::advertisement_builder::{Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList, ServiceUuid16,};

use static_cell::StaticCell;
use embassy_executor::Spawner;

use crate::d_info;

static ADV_DATA: StaticCell<LegacyAdvertisementPayload> = StaticCell::new();
static SCAN_DATA: StaticCell<LegacyAdvertisementPayload> = StaticCell::new();

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[derive(Debug)]
pub enum BLEError {
    NotFound,
    BLEError,
}

type AdvType = Option<&'static LegacyAdvertisementPayload>;
type ScanType = Option<&'static LegacyAdvertisementPayload>;
type SDCfgType = Option<nrf_softdevice::Config>;

pub struct BLEWrapper {
    sd: &'static Softdevice,
    adv_cfg: peripheral::Config,
    adv_data: &'static LegacyAdvertisementPayload,
    scan_data: &'static LegacyAdvertisementPayload,
}

impl BLEWrapper {
    /// One entry point. Binary calls this once.
    pub async fn start(spawner: Spawner, sd_cfg: SDCfgType, adv_data: AdvType , scan_data: ScanType) -> Self {
        // 1. SoftDevice config
        let sd_cfg = match sd_cfg {
            Some(d) => d,
            None => build_default_sd_config(), 
        };

        // 2. Enable SoftDevice
        let sd = Softdevice::enable(&sd_cfg);
        spawner.spawn(softdevice_task(sd)).unwrap();

        // 3. Build payloads (once)
        let adv_data = match adv_data {
            Some(d) => d,
            None => ADV_DATA.init(build_default_adv_payload()),
        };

        let scan_data = match scan_data {
            Some(d) => d,
            None => SCAN_DATA.init(build_default_scan_payload()),
        };

        // 4. Advertising config
        let mut adv_cfg = peripheral::Config::default();
        adv_cfg.interval = 50;

        d_info!("BLE started");

        Self {sd, adv_cfg, adv_data, scan_data,}
    }

    // Non-connectable advertising (basic_ble_advertise)
    pub async fn advertise_nonconnectable(&self) {
        let adv = NonconnectableAdvertisement::ScannableUndirected {
            adv_data: self.adv_data,
            scan_data: self.scan_data,
        };

        peripheral::advertise(self.sd, adv, &self.adv_cfg).await.unwrap();
    }

    // Connectable advertising (ble_char)
    pub async fn advertise_connectable(&self) -> ble::Connection
    {
        let adv = ConnectableAdvertisement::ScannableUndirected {
            adv_data: self.adv_data,
            scan_data: self.scan_data,
        };

        peripheral::advertise_connectable(self.sd, adv, &self.adv_cfg).await.unwrap()
    }

    pub fn get_ble_address(&self) -> Result<String<17>, BLEError> {
        // Print SoftDevice BLE address
        // Should already be a unique address from the FICR (Factory Information Configuration Registers)
        let [h1, h2, h3, h4, h5, h6] = ble::get_address(self.sd).bytes();

        // Create string
        let mut address: String<17> = String::new();
        let _ = write!(address, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", h6, h5, h4, h3, h2, h1);  

        // Log address
        d_info!("BLE Address is {}", address.as_str());

        Ok(address)
    }
}


// DEFAULTS

// Default advertisement payload
fn build_default_adv_payload() -> LegacyAdvertisementPayload {
    LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(ServiceList::Complete, &[ServiceUuid16::BATTERY])
        .short_name("MyShortRust")
        .build()
}

// Default scan payload
fn build_default_scan_payload() -> LegacyAdvertisementPayload {
    LegacyAdvertisementBuilder::new()
        .full_name("MyLongRust")
        .build()
}

// Default SoftDevice config
fn build_default_sd_config() -> nrf_softdevice::Config {
    nrf_softdevice::Config {
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
    }
}