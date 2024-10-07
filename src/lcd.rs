pub mod img;

use defmt::debug;
use embassy_stm32::{gpio::Output, mode::Async, spi::Spi};
use embassy_time::Timer;
use core::{cmp::min, ops::RemAssign};

pub struct LCD {
    /// SPI1外设
    spi: Spi<'static, Async>,
    /// 命令/数据选择
    dc: Output<'static>,
    /// 重置
    res: Output<'static>,
    /// SPI1的NSS
    cs: Output<'static>,
    /// 背光
    blk: Output<'static>,
}

impl LCD {
    pub async fn new(
        spi: Spi<'static, Async>, dc: Output<'static>, res: Output<'static>, cs: Output<'static>,
        blk: Output<'static>,
    ) -> Self {
        let mut lcd = Self {
            spi,
            dc,
            res,
            cs,
            blk,
        };
        lcd.init().await;
        lcd
    }

    async fn init(&mut self) {
        debug!("lcd initing");

        self.res.set_low(); // 复位
        Timer::after_millis(100).await;
        self.res.set_high();
        Timer::after_millis(100).await;

        self.blk.set_high(); // 打开背光
        Timer::after_millis(100).await;

        self.write_reg(0x11).await; // Sleep exit
        Timer::after_millis(120).await; // Delay 120ms

        // Frame Rate Control (In normal mode/ Full colors)
        self.write_reg(0xB1).await;
        self.write_data(0x05).await;
        self.write_data(0x3C).await;
        self.write_data(0x3C).await;

        // Frame Rate Control (In Idle mode/ 8-colors)
        self.write_reg(0xB2).await;
        self.write_data(0x05).await;
        self.write_data(0x3C).await;
        self.write_data(0x3C).await;

        // Frame Rate Control (In Partial mode/ full colors)
        self.write_reg(0xB3).await;
        self.write_data(0x05).await;
        self.write_data(0x3C).await;
        self.write_data(0x3C).await;
        self.write_data(0x05).await;
        self.write_data(0x3C).await;
        self.write_data(0x3C).await;

        // Display Inversion Control
        self.write_reg(0xB4).await; // Dot inversion
        self.write_data(0x03).await;

        // Power Control 1
        self.write_reg(0xC0).await;
        self.write_data(0x0E).await;
        self.write_data(0x0E).await;
        self.write_data(0x04).await;

        // Power Control 2
        self.write_reg(0xC1).await;
        self.write_data(0xC5).await;

        // Power Control 3 (In Normal mode)
        self.write_reg(0xC2).await;
        self.write_data(0x0d).await;
        self.write_data(0x00).await;

        // Power Control 4 (In Idle mode)
        self.write_reg(0xC3).await;
        self.write_data(0x8D).await;
        self.write_data(0x2A).await;

        // Power Control 5 (In Partial mode)
        self.write_reg(0xC4).await;
        self.write_data(0x8D).await;
        self.write_data(0xEE).await;

        // VCOM Control 1
        self.write_reg(0xC5).await; // VCOM
        self.write_data(0x06).await; // 1D  .06

        // Memory Data Access Control
        self.write_reg(0x36).await; // MX, MY, RGB mode
        self.write_data(0x78).await;

        // Interface Pixel Format
        self.write_reg(0x3A).await;
        self.write_data(0x55).await;

        // Gamma (‘+’polarity) Correction Characteristics Setting
        self.write_reg(0xE0).await;
        self.write_data(0x0b).await;
        self.write_data(0x17).await;
        self.write_data(0x0a).await;
        self.write_data(0x0d).await;
        self.write_data(0x1a).await;
        self.write_data(0x19).await;
        self.write_data(0x16).await;
        self.write_data(0x1d).await;
        self.write_data(0x21).await;
        self.write_data(0x26).await;
        self.write_data(0x37).await;
        self.write_data(0x3c).await;
        self.write_data(0x00).await;
        self.write_data(0x09).await;
        self.write_data(0x05).await;
        self.write_data(0x10).await;

        // Gamma ‘-’polarity Correction Characteristics Setting
        self.write_reg(0xE1).await;
        self.write_data(0x0c).await;
        self.write_data(0x19).await;
        self.write_data(0x09).await;
        self.write_data(0x0d).await;
        self.write_data(0x1b).await;
        self.write_data(0x19).await;
        self.write_data(0x15).await;
        self.write_data(0x1d).await;
        self.write_data(0x21).await;
        self.write_data(0x26).await;
        self.write_data(0x39).await;
        self.write_data(0x3E).await;
        self.write_data(0x00).await;
        self.write_data(0x09).await;
        self.write_data(0x05).await;
        self.write_data(0x10).await;

        Timer::after_millis(120).await;
        self.write_reg(0x29).await; // Display on

        debug!("lcd has init");
    }

    async fn write_reg(&mut self, reg: u8) {
        self.dc.set_low();
        self.write_data(reg).await;
        self.dc.set_high();
    }

    async fn write_data(&mut self, data: u8) {
        self.cs.set_low();
        self.spi.write(&[data]).await.unwrap();
        Timer::after_micros(1).await;
        self.cs.set_high();
    }

    async fn write_data_u16(&mut self, data: u16) {
        self.write_data((data >> 8) as u8).await;
        self.write_data(data as u8).await;
    }

    pub async fn fill(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16, color: u16) {
        self.set_address(x_start, y_start, x_end - 1, y_end - 1)
            .await;

        let mut buf = [0u16; 32];
        let mut remain = ((x_end - x_start) * (y_end - y_start)) as usize;

        while remain > 0 {
            let n = min(remain, 32) as usize;
            for i in 0..n {
                buf[i] = color;
            }

            self.cs.set_low();
            self.spi.write(&buf[..n]).await.unwrap();
            Timer::after_micros(1).await;
            self.cs.set_high();

            remain -= n;
        }
    }

    pub async fn fill_img(&mut self) {
        self.set_address(0, 0, 160 - 1, 80 - 1).await;
        self.cs.set_low();
        self.spi.write(&img::G_IMAGE_IMG).await.unwrap();
        Timer::after_micros(1).await;
        self.cs.set_high();
    }

    async fn set_address(&mut self, x_start: u16, y_start: u16, x_end: u16, y_end: u16) {
        self.write_reg(0x2a).await; // 列地址设置
        self.write_data_u16(x_start).await;
        self.write_data_u16(x_end).await;
        self.write_reg(0x2b).await; // 行地址设置
        self.write_data_u16(y_start + 24).await;
        self.write_data_u16(y_end + 24).await;
        self.write_reg(0x2c).await; // 储存器写
    }
}
