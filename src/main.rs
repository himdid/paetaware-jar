#![no_std]
#![no_main]

use atsamd_hal::gpio::Pins;
use atsamd_hal::pac::Peripherals;
use atsamd_hal::prelude::*;
use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let pins = Pins::new(peripherals.port);

    // PA17 is the onboard yellow/orange "L" LED on the XIAO SAMD21 (active-low)
    let mut led = pins.pa17.into_push_pull_output();

    loop {
        led.set_low().unwrap();
        // At default 1 MHz boot clock, ~250,000 cycles is ~0.5 seconds
        asm::delay(250_000);
        led.set_high().unwrap();
        asm::delay(250_000);
    }
}