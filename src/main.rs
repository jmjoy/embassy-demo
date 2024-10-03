#![no_std]
#![no_main]

use core::sync::atomic::{AtomicIsize, Ordering};
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::Timer;
use panic_probe as _;

static NUM: AtomicIsize = AtomicIsize::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("点灯大师，启动!");

    let led = Output::new(p.PC13, Level::High, Speed::VeryHigh);
    let btn = Input::new(p.PA1, Pull::Up);

    spawner.spawn(handle_num(btn, led)).unwrap();

    loop {
        Timer::after_secs(1).await;
        info!("NUM: {}", NUM.load(Ordering::Relaxed));
    }
}

/**
 * 软件消抖
 */
#[embassy_executor::task]
async fn handle_num(btn: Input<'static>, mut led: Output<'static>) {
    let mut last_level = btn.get_level();
    loop {
        Timer::after_millis(20).await;
        let now_level = btn.get_level();
        if now_level == Level::Low && last_level == Level::High {
            led.set_low();
            NUM.fetch_add(1, Ordering::Release);
        }
        if now_level == Level::High && last_level == Level::Low {
            led.set_high();
        }
        last_level = now_level;
    }
}
