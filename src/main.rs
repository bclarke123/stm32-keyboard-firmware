#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::interrupt::free;
use cortex_m::asm::delay as cycle_delay;
use cortex_m::peripheral::NVIC;

use stm32f0xx_hal as hal;
use hal::{prelude::*, pac, pac::interrupt, usb::Peripheral};

use stm32_usbd::bus::UsbBus;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::*;

use usbd_hid_device::USB_CLASS_HID;
use usbd_hid::hid_class::HIDClass;
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus<Peripheral>>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus<Peripheral>>> = None;
static mut USB_HID: Option<HIDClass<UsbBus<Peripheral>>> = None;

#[interrupt]
fn USB() {
    unsafe {
        if let (Some(usb_dev), Some(hid)) = (USB_BUS.as_mut(), USB_HID.as_mut()) {
            usb_dev.poll(&mut [hid]);
        }
    };
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(mut cp)) = (pac::Peripherals::take(), pac::CorePeripherals::take()) {
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

        let usb_bus = unsafe {
            USB_ALLOCATOR = Some(
                UsbBus::new(Peripheral {
                    usb: p.USB,
                    pin_dm,
                    pin_dp
                })
            );
            USB_ALLOCATOR.as_ref().unwrap()
        };

        unsafe {
            USB_HID = Some(HIDClass::new(&usb_bus, KeyboardReport::desc(), 60));
            USB_BUS = Some(UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xbe17, 0xc1a2))
                .manufacturer("Ben Custom")
                .product("Macro Breadboard")
                .serial_number("TEST")
                .device_class(USB_CLASS_HID)
                .build());
        }

        unsafe {
            cp.NVIC.set_priority(interrupt::USB, 1);
            NVIC::unmask(interrupt::USB);
        }

        for led in &mut leds {
            led.set_low().unwrap();
        }

        let mut prev_codes = [0u8; 6];
        let mut prev_mod = 0u8;

        let mut report = KeyboardReport {
            modifier: 0,
            leds: 0,
            reserved: 0,
            keycodes: [ 0u8; 6 ]
        };

        loop {

            cycle_delay(1024 * 1024);

            let mut led_idx = 0;
            let mut code_idx = 0;

            report.modifier = 0;
            report.keycodes.fill(0);

            for col in cols.iter_mut() {

                col.set_high().unwrap();

                for row in rows.iter() {

                    let high = row.is_high().unwrap();
                    leds[led_idx].set_state(high.into()).unwrap();

                    if high {
                        report.keycodes[code_idx] = [
                            0x05,
                            0x08,
                            0x11,
                            0x1e,
                        ][led_idx];

                        report.modifier = [
                            0, 0, 0, 0x02
                        ][led_idx];

                        code_idx += 1;
                    }

                    led_idx += 1;

                }

                col.set_low().unwrap();

            }

            if report.keycodes == prev_codes && report.modifier == prev_mod {
                continue;
            }

            prev_codes[..].copy_from_slice(&report.keycodes);
            prev_mod = report.modifier;

            match free(|_| unsafe { USB_HID.as_mut().map(|hid| hid.push_input(&report)) })
                .unwrap() {
                    Ok(_) => {},
                    _ => {
                        leds[1].set_high().unwrap();
                        leds[2].set_high().unwrap();
                    }
                }
        }
    }

    loop {
        continue;
    }
}
