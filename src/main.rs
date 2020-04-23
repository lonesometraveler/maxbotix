#![no_main]
#![no_std]

extern crate panic_halt;

use core::fmt::Write;
use heapless::consts::*;
use heapless::String;

use cortex_m;
use cortex_m::{iprintln, peripheral};
use cortex_m_rt::entry;
use max6955::*;
use maxsonar::{MaxSonar, Model};
use stm32f4xx_hal::{i2c::I2c, prelude::*, stm32};

fn itm() -> &'static mut peripheral::itm::Stim {
    unsafe { &mut (*peripheral::ITM::ptr()).stim[0] }
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Set up LED Display
    let gpiob = dp.GPIOB.split();
    let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
    let sda = gpiob.pb9.into_alternate_af4().set_open_drain();
    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 10.khz(), clocks);
    let mut max6955 = Max6955::new(i2c).unwrap();
    max6955.powerup().unwrap();
    max6955.set_global_intensity(6).unwrap();

    // Set up Sonar
    let gpioc = dp.GPIOC.split();
    let pin = gpioc.pc10.into_pull_down_input();
    let sysclk = clocks.sysclk();
    let mut sonar = MaxSonar::new(dp.TIM2, Model::LV, pin, sysclk);

    let mut data = String::<U8>::new();
    let mut last_distance = 0u32;

    loop {
        let distance = sonar.read();
        if distance == last_distance {
            continue;
        }

        last_distance = distance;
        let _ = write!(data, "{:7}{}", distance, sonar.unit());
        max6955.write_str(&data).unwrap();
        data.clear();
        iprintln!(itm(), "{}{}", distance, sonar.unit());
    }
}
