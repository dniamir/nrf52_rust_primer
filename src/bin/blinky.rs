// Blinky
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use nrf52_rust_primer::{self as _, led::Led};
use nrf52_rust_primer::{dlogger::DLogger, d_info};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut led = Led::new(p.P0_13);
    
    d_info!("Blinky started!");

    let mut count = 0;

    loop {
        count += 1;

        d_info!("Count: {}", count);
        led.blink(100).await;
        DLogger::d_sep();
    }
}
