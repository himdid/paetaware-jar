#![no_std]
#![no_main]

mod display;
mod sprites;

use atsamd_hal::gpio::Pins;
use atsamd_hal::pac::Peripherals;
use atsamd_hal::prelude::*;
use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;

use display::Screen;
use sprites::{PET_IDLE_0, PET_IDLE_1};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let pins = Pins::new(peripherals.port);

    let mut led = pins.pa17.into_push_pull_output();
    led.set_high().unwrap();

    let mut screen = Screen::new(
        pins.pa08.into_push_pull_output(),
        pins.pa09.into_push_pull_output(),
    );

    let mut frame: u8 = 0;

    loop {
        screen.clear();
        let sprite = if frame == 0 { &PET_IDLE_0 } else { &PET_IDLE_1 };
        screen.draw_sprite_2x(sprite, 48, 16);
        screen.flush();

        frame ^= 1;

        led.set_low().unwrap();
        asm::delay(250_000);
        led.set_high().unwrap();
        asm::delay(250_000);
    }
}