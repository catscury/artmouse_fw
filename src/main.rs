//! CDC-ACM serial port example using polling in a busy loop.
//! Target board: Blue Pill
#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::pac;
use stm32f1xx_hal::{prelude::*, spi::*};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use numtoa::NumToA;
mod error;

use embedded_nrf24l01 as nrf24;
use nrf24::{Configuration, NRF24L01};

fn main_err() -> Result<(), error::AppError> {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Configure the on-board LED (PC13, green)
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_low()?;

    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa6.into_floating_input(&mut gpioa.crl),
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
    );

    let csn = gpioa.pa2.into_push_pull_output(&mut gpioa.crl);
    let ce = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);

    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };
    let spi = Spi::spi1(dp.SPI1, pins, &mut afio.mapr, spi_mode, 2000.khz(), clocks, &mut rcc.apb2);

    let mut state_nrf24 = NRF24L01::new(ce, csn, spi)?;
    state_nrf24.set_frequency(8)?;
    state_nrf24.set_auto_retransmit(0, 0)?;
    state_nrf24.set_rf(&nrf24::DataRate::R2Mbps, 3)?;
    state_nrf24.set_pipes_rx_enable(&[true, false, false, false, false, false])?;
    state_nrf24.set_pipes_rx_lengths(&[Some(32u8), None, None, None, None, None])?;
    state_nrf24.set_auto_ack(&[false; 6])?;
    state_nrf24.set_crc(nrf24::CrcMode::Disabled)?;
    state_nrf24.set_rx_addr(0, &b"asded"[..])?;
    state_nrf24.set_tx_addr(&b"asded"[..])?;

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low()?;
    delay(clocks.sysclk().0/100);
    led.set_low()?;

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();


    let mut rx_nrf24 = state_nrf24.rx().unwrap();
    let mut buf = [0u8; 64];
    let mut buf_c = [0u8; 10];
    let mut decim = 0;
    let decim_mask = (1<<14)-1;

    loop {
        let usb_avail = usb_dev.poll(&mut [&mut serial]);
        let nrf_avail = match rx_nrf24.can_read()? {
            Some(_) => true,
            None => false,
        };

        if usb_avail {
            match serial.read(&mut buf) {
                Ok(count) if count > 2 => {
                    led.set_low()?; // Turn on

                    let mut tx_nrf24 = rx_nrf24
                        .standby()
                        .tx()
                        .unwrap();
                    tx_nrf24.send(&buf[0..count])?;
                    tx_nrf24.wait_empty()?;
                    rx_nrf24 = tx_nrf24
                        .standby()
                        .unwrap()
                        .rx()
                        .unwrap();
                }
                _ => {}
            }
        }

        if nrf_avail {
            led.set_low()?;
            let recv = rx_nrf24.read()?;
            serial.write(&recv)?;
        }

        if decim & decim_mask == 0  {
            serial.write(b"Freq ")?;
            serial.write(rx_nrf24.get_frequency()?.numtoa(10, &mut buf_c))?;
            serial.write(b" ")?;
            serial.write(rx_nrf24.get_address_width()?.numtoa(10, &mut buf_c))?;
            serial.write(b" \r\n ")?;
        }
        decim = decim + 1;

        led.set_high()?; // Turn off
    }
}


#[entry]
fn main() -> ! {
    match main_err() {
        Ok(_) => 0,
        Err(x) => x.code,
    };
    loop {}
}
