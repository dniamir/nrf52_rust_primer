#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;

// use nrf52_rust_primer as _;
use nrf52_rust_primer::{self as _, info};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    info!("Blinky started!");

    let mut count = 0;

    loop {
        count += 1;

        info!("Count: {}", count);
        led.set_high();
        Timer::after_millis(500).await;
        info!("LED ON");
        led.set_low();
        Timer::after_millis(500).await;
        info!("LED OFF");
        info!("========");
    }
}