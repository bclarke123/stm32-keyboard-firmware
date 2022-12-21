#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{delay::Delay, pac, prelude::*};

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
        let gpiob = p.GPIOB.split(&mut rcc);
        let mut led = cortex_m::interrupt::free(|cs| gpiob.pb3.into_push_pull_output(cs));
        let mut delay = Delay::new(cp.SYST, &rcc);

        loop {
            led.toggle().ok();
            delay.delay_ms(2000_u16);
        }
    }

    loop {
        continue;
    }
}
