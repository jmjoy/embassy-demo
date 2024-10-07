#![no_std]
#![no_main]

mod lcd;
mod w25q64;
mod w25q64_hal;

use core::sync::atomic::{AtomicIsize, Ordering};
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Input, Level, Output, Pull, Speed},
    mode::Async,
    pac,
    spi::{self, Spi},
    time, usart::{self, Uart, UartTx},
};
use embassy_time::Timer;
use embedded_hal::{digital::OutputPin, spi::SpiDevice};
use embedded_hal_async::spi::SpiBus;
use lcd::LCD;
use panic_probe as _;
use w25q64::W25Q64;
use w25q64_hal::W25Q64Hal;

static NUM: AtomicIsize = AtomicIsize::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("点灯大师，启动!");

    // BTN
    let led = Output::new(p.PC13, Level::High, Speed::VeryHigh);
    let btn = Input::new(p.PA1, Pull::Up);
    spawner.spawn(handle_num(btn, led)).unwrap();

    // W25Q64
    let nss = Output::new(p.PB12, Level::High, Speed::VeryHigh);
    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::MODE_0;
    spi_config.bit_order = spi::BitOrder::MsbFirst;
    let spi = Spi::new(
        p.SPI2, p.PB13, p.PB15, p.PB14, p.DMA1_CH5, p.DMA1_CH4, spi_config,
    );
    spawner.spawn(w25q46_hal_task(spi, nss)).unwrap();

    // LCD
    pac::AFIO.mapr().modify(|w| {
        w.set_swj_cfg(0b0000_0010); // this is equal to __HAL_AFIO_REMAP_SWJ_NOJTAG() in C
        w.set_spi1_remap(true);
    });
    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::MODE_3;
    spi_config.bit_order = spi::BitOrder::MsbFirst;
    spi_config.frequency = time::mhz(8 / 2);
    let spi = Spi::new_txonly(p.SPI1, p.PB3, p.PB5, p.DMA1_CH3, spi_config);
    let dc = Output::new(p.PB4, Level::High, Speed::VeryHigh);
    let res = Output::new(p.PB6, Level::High, Speed::VeryHigh);
    let cs = Output::new(p.PB7, Level::High, Speed::VeryHigh);
    let blk = Output::new(p.PB8, Level::High, Speed::VeryHigh);
    spawner.spawn(show_lcd(spi, dc, res, cs, blk)).unwrap();

    // Print
    loop {
        Timer::after_secs(1).await;
        info!("NUM: {}", NUM.load(Ordering::Relaxed));
    }
}

/// 软件消抖
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

#[embassy_executor::task]
async fn show_lcd(
    spi: Spi<'static, Async>, dc: Output<'static>, res: Output<'static>, cs: Output<'static>,
    blk: Output<'static>,
) {
    let mut lcd = LCD::new(spi, dc, res, cs, blk).await;
    lcd.fill(0, 0, 160, 80, 0xFC07).await;
    lcd.fill_img().await;
    loop {
        Timer::after_secs(6).await;
    }
}

#[embassy_executor::task]
async fn w25q46_task(spi: Spi<'static, Async>, nss: Output<'static>) {
    let mut w25q64 = W25Q64::new(spi, nss);
    let jedec = w25q64.read_jedec().await;
    info!(
        "w25q64 jedec: vendor: {}, device: {}",
        jedec.vendor_id, jedec.device_id
    );
}

#[embassy_executor::task]
async fn w25q46_hal_task(spi: impl SpiBus + 'static, nss: impl OutputPin + 'static) {
    let mut w25q64 = W25Q64Hal::new(spi, nss);
    let jedec = w25q64.read_jedec().await;
    info!(
        "w25q64 jedec: vendor: {}, device: {}",
        jedec.vendor_id, jedec.device_id
    );
}
