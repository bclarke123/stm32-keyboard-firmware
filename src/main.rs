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

        let col1 = free(|cs| gpioa.pa0.into_push_pull_output(cs)).downgrade();
        let col2 = free(|cs| gpioa.pa1.into_push_pull_output(cs)).downgrade();
        let row1 = free(|cs| gpioa.pa3.into_pull_down_input(cs)).downgrade();
        let row2 = free(|cs| gpioa.pa4.into_pull_down_input(cs)).downgrade();
        let led1 = free(|cs| gpioa.pa5.into_push_pull_output(cs)).downgrade();
        let led2 = free(|cs| gpioa.pa6.into_push_pull_output(cs)).downgrade();
        let led3 = free(|cs| gpioa.pa2.into_push_pull_output(cs)).downgrade();
        let led4 = free(|cs| gpioa.pa7.into_push_pull_output(cs)).downgrade();

        let mut cols = [ col1, col2 ];
        let rows = [ row1, row2 ];

        let mut leds = [ led1, led2, led3, led4 ];

        let mut delay = Delay::new(cp.SYST, &rcc);

        loop {

            let mut led_idx = 0;

            for col in cols.iter_mut() {

                col.set_high().unwrap();

                for row in rows.iter() {

                    let high = row.is_high().unwrap();
                    leds[led_idx].set_state(high.into()).unwrap();

                    led_idx += 1;

                }

                col.set_low().unwrap();

            }

            delay.delay_ms(50_u16);

            continue;
        }
    }

    loop {
        continue;
    }
}
