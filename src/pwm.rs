use defmt::debug;
use embassy_stm32::{
    gpio::OutputType,
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
        Channel, Channel1Pin, Channel2Pin, Channel3Pin, GeneralInstance4Channel,
    },
    Peripheral,
};

pub struct RgbLed<'d, T: GeneralInstance4Channel> {
    pwm: SimplePwm<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> RgbLed<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd, pin1: impl Peripheral<P = impl Channel1Pin<T>> + 'd,
        pin2: impl Peripheral<P = impl Channel2Pin<T>> + 'd,
        pin3: impl Peripheral<P = impl Channel3Pin<T>> + 'd,
    ) -> Self {
        let pwm = SimplePwm::new(
            tim,
            Some(PwmPin::new_ch1(pin1, OutputType::PushPull)),
            Some(PwmPin::new_ch2(pin2, OutputType::PushPull)),
            Some(PwmPin::new_ch3(pin3, OutputType::PushPull)),
            None,
            Hertz::khz(10),
            CountingMode::EdgeAlignedUp,
        );

        debug!("PWM max duty: {}", pwm.get_max_duty());

        Self { pwm }
    }

    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        let max_duty = self.pwm.get_max_duty();

        self.pwm.set_duty(
            Channel::Ch1,
            (u8::MAX - r) as u32 * max_duty / u8::MAX as u32,
        );
        self.pwm.set_duty(
            Channel::Ch2,
            (u8::MAX - g) as u32 * max_duty / u8::MAX as u32,
        );
        self.pwm.set_duty(
            Channel::Ch3,
            (u8::MAX - b) as u32 * max_duty / u8::MAX as u32,
        );

        self.pwm.enable(Channel::Ch1);
        self.pwm.enable(Channel::Ch2);
        self.pwm.enable(Channel::Ch3);
    }
}
