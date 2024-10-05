use embassy_stm32::{gpio::Output, mode::Async, spi::Spi};

#[derive(Default)]
pub struct Jedec {
    pub vendor_id: u8,
    pub device_id: u16,
}

pub struct W25Q64 {
    spi: Spi<'static, Async>,
    nss: Output<'static>,
}

impl W25Q64 {
    pub fn new(spi: Spi<'static, Async>, nss: Output<'static>) -> Self {
        Self { spi, nss }
    }

    pub async fn read_jedec(&mut self) -> Jedec {
        let mut jedec = Jedec::default();

        let mut buf = [0u8];

        self.nss.set_low(); // MY_SPI_START();
        self.spi.transfer(&mut buf, &[0x9F]).await.unwrap(); // MY_SPI_SWAP(0x9F);
        self.spi.transfer(&mut buf, &[0xFF]).await.unwrap(); // jedec->vendor_id = MY_SPI_SWAP(0xFF);
        jedec.vendor_id = buf[0];
        self.spi.transfer(&mut buf, &[0xFF]).await.unwrap(); // jedec->device_id = MY_SPI_SWAP(0xFF);
        jedec.device_id = (buf[0] as u16) << 8; // jedec->device_id <<= 8;
        self.spi.transfer(&mut buf, &[0xFF]).await.unwrap(); // jedec->device_id |= MY_SPI_SWAP(0xFF);
        jedec.device_id |= buf[0] as u16; // jedec->device_id <<= 8;
        self.nss.set_high(); // MY_SPI_STOP();

        jedec
    }
}
