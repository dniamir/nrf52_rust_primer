#![no_std]
#![no_main]

use embassy_executor::Spawner;
use nrf52_rust_primer::{self as _, info, led::Led};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut led = Led::new(p.P0_13);
    
    info!("Blinky started!");

    let mut count = 0;

    loop {
        count += 1;

        info!("Count: {}", count);
        led.blink(100).await;
        info!("========");
    }
}
