#![no_std]
#![no_main]

mod display;
mod face;
mod input;
mod pet;
mod protocol;

use atsamd_hal::gpio::Pins;
use atsamd_hal::pac::Peripherals;
use atsamd_hal::prelude::*;
use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;

use display::Screen;
use input::{InputEvent, Inputs};
use pet::Pet;
use protocol::DockLink;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
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
        pins.pa05.into_pull_up_input(), // D9: Standby slide switch
    );

    let mut dock = DockLink::new(
        peripherals.sercom4,
        &mut peripherals.pm,
        &mut peripherals.gclk,
        pins.pb08.into_alternate(), // D6: SERCOM4 TX
        pins.pb09.into_alternate(), // D7: SERCOM4 RX
    );

    let mut pet = Pet::new();
    let mut anim_counter: u16 = 0;
    let mut tx_counter: u8 = 0;

    pet.set_sleeping(inputs.is_sleep_switch_on());
    dock.send_telemetry(&pet.to_payload());
    screen.render(&pet);

    loop {
        if let Some(pkt) = dock.poll() {
            pet.handle_packet(pkt);
            screen.render(&pet);
        }

        if pet.set_sleeping(inputs.is_sleep_switch_on()) {
            dock.send_telemetry(&pet.to_payload());
            screen.render(&pet);
        }

        if let Some(event) = inputs.poll() {
            match event {
                InputEvent::Left => pet.move_left(),
                InputEvent::Right => pet.move_right(),
                InputEvent::Confirm => pet.confirm(),
                InputEvent::Cancel => pet.cancel(),
                InputEvent::Step => pet.record_step(),
            }
            dock.send_telemetry(&pet.to_payload());
            screen.render(&pet);
        }

        anim_counter += 1;
        if anim_counter >= 180 {
            anim_counter = 0;
            pet.tick();
            screen.render(&pet);

            tx_counter += 1;
            if tx_counter >= 20 {
                tx_counter = 0;
                dock.send_telemetry(&pet.to_payload());
                let _ = led.toggle();
            }
        }

        asm::delay(1_000);
    }
}