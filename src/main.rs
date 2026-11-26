#![no_std]
#![no_main]

mod audio;
mod display;
mod input;
mod pet;
mod sprites;

use atsamd_hal::gpio::Pins;
use atsamd_hal::pac::Peripherals;
use atsamd_hal::prelude::*;
use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;

use audio::Buzzer;
use display::Screen;
use input::{InputEvent, Inputs};
use pet::Pet;

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

    let mut inputs = Inputs::new(
        pins.pa02.into_pull_up_input(), // D0: Left
        pins.pa04.into_pull_up_input(), // D1: Right
        pins.pa10.into_pull_up_input(), // D2: Confirm (A)
        pins.pa11.into_pull_up_input(), // D3: Cancel (B)
        pins.pa06.into_pull_up_input(), // D10: Pedometer step switch
    );

    let mut buzzer = Buzzer::new(pins.pa07.into_push_pull_output()); // D8: Passive Buzzer

    let mut pet = Pet::new();
    let mut frame: u8 = 0;
    let mut tick_counter: u32 = 0;

    screen.render(&pet, frame);
    buzzer.beep_confirm();

    loop {
        if let Some(event) = inputs.poll() {
            match event {
                InputEvent::Left => {
                    pet.move_left();
                    buzzer.beep_move();
                }
                InputEvent::Right => {
                    pet.move_right();
                    buzzer.beep_move();
                }
                InputEvent::Confirm => {
                    pet.confirm();
                    buzzer.beep_confirm();
                }
                InputEvent::Cancel => {
                    pet.cancel();
                    buzzer.beep_cancel();
                }
                InputEvent::Step => {
                    pet.record_step();
                    if pet.steps % 10 == 0 {
                        buzzer.beep_milestone();
                    } else {
                        buzzer.beep_step();
                    }
                }
            }
            screen.render(&pet, frame);
        }

        tick_counter += 1;
        if tick_counter >= 2_500 {
            tick_counter = 0;
            frame ^= 1;
            pet.tick();
            screen.render(&pet, frame);
            let _ = led.toggle();
        }

        asm::delay(2_000);
    }
}