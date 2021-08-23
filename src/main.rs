//! Blinks an LED
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
//use embedded_nrf24l01 as state_nrf24;
//use state_nrf24::{Configuration, NRF24L01};
use embedded_hal::digital::v2::{OutputPin, PinState};
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};
mod error;

fn main_err() -> Result<(), error::AppError> {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m:: Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    //let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    //let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);


    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());


    // let pins = (
    //     gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
    //     gpioa.pa6.into_floating_input(&mut gpioa.crl),
    //     gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
    // );

    // let ce = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    // let csn = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

    // let spi_mode = Mode {
    //     polarity: Polarity::IdleLow,
    //     phase: Phase::CaptureOnFirstTransition,
    // };
    // let spi = Spi::spi1(dp.SPI1, pins, &mut afio.mapr, spi_mode, 100.khz(), clocks, &mut rcc.apb2);

    // let mut state_nrf24 = NRF24L01::new(ce, csn, spi)?;

    // state_nrf24.set_frequency(8)?;
    // state_nrf24.set_auto_retransmit(0, 0)?;
    // state_nrf24.set_rf(&state_nrf24::DataRate::R250Kbps, 3)?;
    // state_nrf24.set_pipes_rx_enable(&[true, false, false, false, false, false])?;
    // state_nrf24.set_auto_ack(&[false; 6])?;
    // state_nrf24.set_crc(state_nrf24::CrcMode::Disabled)?;
    // state_nrf24.set_rx_addr(0, &b"base"[..])?;
    // state_nrf24.set_tx_addr(&b"mouse"[..])?;

    let mut led_state = PinState::Low;
    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        //let mut rx_nrf24 = state_nrf24.rx().unwrap();
        block!(timer.wait()).unwrap();
        //block!(timer.wait()).unwrap();
        //block!(timer.wait()).unwrap();
        //if !rx_nrf24.is_empty()? {
          //  rx_nrf24.read()?;
            led_state = !led_state;
            led.set_state(led_state)?;
        //}
        //state_nrf24 = rx_nrf24.standby();
        //let mut tx_nrf24 = state_nrf24.tx().unwrap();
        //tx_nrf24.send(&b"shit"[..])?;
        //tx_nrf24.wait_empty()?;
        //state_nrf24 = tx_nrf24.standby()?;
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
