#![no_std]

use stm32f4xx_hal::hal::digital::v2::InputPin;
use stm32f4xx_hal::stm32::{RCC, TIM2};
use stm32f4xx_hal::time::Hertz;

pub struct MaxSonar<T> {
    timer: TIM2,
    model: Model,
    pin: T,
}

impl<T, E> MaxSonar<T>
where
    T: InputPin<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(timer: TIM2, model: Model, pin: T, sysclk: Hertz) -> Self {
        // Configure timer for 1Mhz
        let rcc = unsafe { &(*RCC::ptr()) };
        rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());
        let psc = (sysclk.0 / 1_000_000) as u16;
        timer.psc.write(|w| w.psc().bits(psc - 1));
        timer.egr.write(|w| w.ug().set_bit());
        // Start MaxSonar
        let mut sonar = MaxSonar { timer, model, pin };
        sonar.start();
        sonar
    }
    /// Calculates the distance
    pub fn read(&mut self) -> u32 {
        while self.pin.is_low().unwrap() {}
        self.timer.cnt.reset();
        while self.pin.is_high().unwrap() {}
        self.timer.cnt.read().bits() / self.model.factor()
    }
    /// Returns the unit for the model
    pub fn unit(&self) -> &str {
        self.model.unit()
    }
    /// Starts the timer
    fn start(&mut self) {
        self.timer.cnt.reset();
        self.timer.cr1.write(|w| w.cen().set_bit());
    }
}

/// Maxbotix Ultra Sensor Models
#[derive(Debug, Clone, Copy)]
pub enum Model {
    LV,
    XL,
    HR,
}

impl Model {
    /// scale factor
    fn factor(self) -> u32 {
        match self {
            Model::LV => 147,
            Model::XL => 58,
            Model::HR => 1,
        }
    }
    /// unit
    fn unit(self) -> &'static str {
        match self {
            Model::LV => "\"",
            Model::XL => "cm",
            Model::HR => "mm",
        }
    }
}
