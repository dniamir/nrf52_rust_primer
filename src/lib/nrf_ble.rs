// Wrapper for BLE
use heapless::String;
use core::fmt::Write;
use core::mem;
use static_cell::StaticCell;

use nrf_softdevice::ble::advertisement_builder::{Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList, ServiceUuid16,};
use nrf_softdevice::{ble, raw, Softdevice};

use crate::d_info;  // Logging

static ADV_CELL: StaticCell<ble::peripheral::NonconnectableAdvertisement<'static>> = StaticCell::new();
static ADV_DATA_CELL: StaticCell<LegacyAdvertisementPayload> = StaticCell::new();
static SCAN_DATA_CELL: StaticCell<LegacyAdvertisementPayload> = StaticCell::new();

/// Define some error types
#[derive(Debug)]
pub enum BLEError {
    NotFound,
    BLEError,
}

// Struct definition
pub struct BLEWrapper {
    pub sd_config: nrf_softdevice::Config,      // SoftDevice config
    pub sd: Option<&'static Softdevice>,
    pub adv_config: ble::peripheral::Config,    // Advertisement config
    pub adv: Option<ble::peripheral::NonconnectableAdvertisement<'static>>
}

// MUTEX implementations for I2C generic - Any MAP
impl BLEWrapper 
{
    pub fn create_default() -> Result<Self, BLEError> {

        let empty_sd_config = nrf_softdevice::Config::default();
        let empty_adv_config = ble::peripheral::Config::default();
        let this = Self {sd_config: empty_sd_config, sd: None, adv_config: empty_adv_config, adv: None};
        Ok(this)
    }

    pub fn enable(&mut self) -> Result<(), BLEError> {
        // Enable the basic BLE config
        self.sd = Some(Softdevice::enable(&self.sd_config));
        Ok(())
    }

    pub fn set_default_sd_config(&mut self) -> Result<(), BLEError> {
        // Set default BLE config based on example I found

        let default_config = nrf_softdevice::Config {
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

        self.sd_config = default_config;

        Ok(())
    }

    pub fn get_ble_address(&self) -> Result<String<17>, BLEError> {
        // Print SoftDevice BLE address
        // Should already be a unique address from the FICR (Factory Information Configuration Registers)
        let sd_temp = self.sd.as_ref().ok_or(BLEError::NotFound)?;
        let [h1, h2, h3, h4, h5, h6] = ble::get_address(sd_temp).bytes();

        // Create string
        let mut address: String<17> = String::new();
        let _ = write!(address, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", h6, h5, h4, h3, h2, h1);  

        // Log address
        d_info!("BLE Address is {}", address.as_str());

        Ok(address)
    }

    pub fn set_default_adv_config(&mut self, short_name: &str, full_name: &str) -> Result<(), BLEError> {

        // Set advertisement config (limited info)
        self.adv_config = ble::peripheral::Config::default();

        // Set data to advertise
        let adv_data = ADV_DATA_CELL.init(
        LegacyAdvertisementBuilder::new()
            .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
            .services_16(ServiceList::Complete, &[ServiceUuid16::HEALTH_THERMOMETER])
            .short_name(short_name)
            .build()
        );

        // Set data when scanned by another device (more detailed info)
        let scan_data = SCAN_DATA_CELL.init(
            LegacyAdvertisementBuilder::new()
                .full_name(full_name)
                .build()
        );

        // Set up advertisement
        let adv = ADV_CELL.init(
            ble::peripheral::NonconnectableAdvertisement::ScannableUndirected {
                adv_data,
                scan_data,
            }
        );

        self.adv = Some(*adv);

        Ok(())

    }

    pub async fn advertise(&self) -> Result<(), ble::peripheral::AdvertiseError> {
        ble::peripheral::advertise(
            self.sd.unwrap(), 
            self.adv.unwrap(), 
            &self.adv_config,
        ).await?;
        Ok(())
    }
}
