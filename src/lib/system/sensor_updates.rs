/// Setup I2C and periodically update sensor atomics
use core::sync::atomic::Ordering;
use static_cell::StaticCell;

use embassy_time::Timer;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use embassy_hal_internal::Peri;

use crate::embassy_hal::gpio::Pin;
use crate::embassy_hal::{self, Peripherals, bind_interrupts, interrupt::Priority, twim::{self, Twim}};
use crate::embassy_hal::peripherals;
use crate::d_peripherals::chip_implementations::I2CMutexWrapper;
use crate::d_peripherals::sensors::bme680::BME680;

use crate::system::state::{TEMP_VAL, PRESSURE_VAL};
use crate::{d_log::dlogger::DLogger, d_info};

bind_interrupts!(struct Irqs {TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;});
static I2C_MUTEX: StaticCell<Mutex<ThreadModeRawMutex, Twim<'static>>> = StaticCell::new();
static TX_BUF: StaticCell<[u8; 32]> = StaticCell::new();

#[derive(Debug)]
pub enum SensorUpdateError {
    NotFound,
    // PeripheralStartError,
    // I2CStartError,
    // BME680Error(BME680Error),
}


// Initiate peripherals
// Very finicky - HAL interrupts have to be given lower priority than softdeivce
// this block needs to come before SoftDevice is enabled
pub fn start_peripherals() -> Peripherals {
    let mut ecfg = embassy_hal::config::Config::default();
    ecfg.gpiote_interrupt_priority = Priority::P2;
    ecfg.time_interrupt_priority = Priority::P2; // for time-driver-rtc1
    let p = embassy_hal::init(ecfg);

    p
}

// Initalize I2C
pub fn start_i2c<SCL, SDA>(scl: Peri<'static, SCL>, sda: Peri<'static, SDA>, twi: Peri<'static, peripherals::TWISPI0>,) -> I2CMutexWrapper
where
    SCL: Pin,
    SDA: Pin,
{
    // Initialize I2C bus config
    let mut config = twim::Config::default();
    config.frequency = twim::Frequency::K100;
    
    // Initialize I2C bus
    let tx_buf = TX_BUF.init([0u8; 32]);
    let i2c_bus = Twim::new(twi, Irqs, sda, scl, config, tx_buf);
    let i2c_mutex = I2C_MUTEX.init(Mutex::new(i2c_bus));
    let i2c_mutex_wrapper = I2CMutexWrapper(i2c_mutex);

    i2c_mutex_wrapper
}

// Async bme680 reads
#[embassy_executor::task]
pub async fn bme_update(i2c_bus: I2CMutexWrapper, delay_ms: u64) {

    // Do some simple chip reads
    d_info!("Setting up BME680");

    let mut bme = BME680::new(i2c_bus, 0x76).await.unwrap();
    bme.config(1).await.unwrap();
    loop {

        // Read register with generic register read
        bme.chip.read_field("chip_id").await.unwrap();
        bme.chip.read_reg(0xD0).await.unwrap();

        let temp_val = bme.read_temperature().await.unwrap();
        let pressure_val = bme.read_pressure().await.unwrap();

        DLogger::d_sep();

        // Send data to channel
        TEMP_VAL.store(temp_val, Ordering::Relaxed);
        PRESSURE_VAL.store(pressure_val, Ordering::Relaxed);

        // Wait before next scan
        Timer::after_millis(delay_ms).await;
    }

}
