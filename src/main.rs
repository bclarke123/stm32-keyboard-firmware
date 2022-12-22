#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::{ interrupt::free };

use stm32f0xx_hal as hal;
use hal::{prelude::*, pac, usb::Peripheral};

use stm32_usbd::bus::UsbBus;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// use usbd_hid_device::{USB_CLASS_HID, Hid};

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let mut rcc = p.RCC.configure()
        .hsi48()
        .enable_crs(p.CRS)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        let pin_dm = gpioa.pa11;
        let pin_dp = gpioa.pa12;

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

        let peripheral = Peripheral {
            usb: p.USB,
            pin_dm,
            pin_dp
        };

        let usb_bus = UsbBus::new(peripheral);
        let mut serial = SerialPort::new(&usb_bus);

        let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Ben Custom")
            .product("Macro Breadboard")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        loop {

            usb_dev.poll(&mut [ &mut serial ]);

            let mut led_idx = 0;
            let mut buf = [0u8; 5];
            let mut send = false;

            for col in cols.iter_mut() {

                col.set_high().unwrap();

                for row in rows.iter() {

                    let high = row.is_high().unwrap();
                    leds[led_idx].set_state(high.into()).unwrap();
                    if high {
                        buf[led_idx] = [ 'a', 's', 'd', 'f' ][led_idx] as u8;
                        send = true;
                    }

                    led_idx += 1;

                }

                col.set_low().unwrap();

            }

            buf[4] = '\n' as u8;

            if send {
                let mut write_offset = 0;
                while write_offset < 4 {
                    match serial.write(&buf[write_offset..]) {
                        Ok(len) if len > 0 => {
                            write_offset += len
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    loop {
        continue;
    }
}
