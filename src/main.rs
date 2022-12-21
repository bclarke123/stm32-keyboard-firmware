#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use hal::{delay::Delay, pac, prelude::*};
use cortex_m::{ interrupt::free, peripheral::Peripherals };
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (pac::Peripherals::take(), Peripherals::take()) {
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
        let gpioa = p.GPIOA.split(&mut rcc);
        let gpiob = p.GPIOB.split(&mut rcc);
        let mut led = free(|cs| gpiob.pb3.into_push_pull_output(cs));
        let mut led2 = free(|cs| gpioa.pa1.into_push_pull_output(cs));
        let mut delay = Delay::new(cp.SYST, &rcc);

        led2.toggle().ok();

        loop {
            delay.delay_ms(2000_u16);
            led.toggle().ok();
            led2.toggle().ok();
        }
    }

    loop {
        continue;
    }
}
