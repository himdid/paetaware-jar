pub mod display;
pub mod input;
pub mod uart;

use atsamd_hal::gpio::{Pin, Pins, PushPullOutput, PA17};
use atsamd_hal::pac::Peripherals;
use atsamd_hal::prelude::*;
use cortex_m::asm;

use crate::platform::{InputEvent, PetPlatform, RenderScene};
use display::Screen;
use input::Inputs;
use uart::XiaoUart;

pub struct XiaoBoard {
    led: Pin<PA17, PushPullOutput>,
    screen: Screen,
    inputs: Inputs,
    uart: XiaoUart,
}

impl XiaoBoard {
    pub fn init(mut peripherals: Peripherals) -> Self {
        let pins = Pins::new(peripherals.port);

        let mut led = pins.pa17.into_push_pull_output();
        led.set_high().unwrap();

        let screen = Screen::new(
            pins.pa08.into_push_pull_output(),
            pins.pa09.into_push_pull_output(),
        );

        let inputs = Inputs::new(
            pins.pa02.into_pull_up_input(), // D0: Left[cite: 4]
            pins.pa04.into_pull_up_input(), // D1: Right[cite: 4]
            pins.pa10.into_pull_up_input(), // D2: Confirm (A)[cite: 4]
            pins.pa11.into_pull_up_input(), // D3: Cancel (B)[cite: 4]
            pins.pa06.into_pull_up_input(), // D10: Pedometer step switch[cite: 4]
            pins.pa05.into_pull_up_input(), // D9: Standby slide switch[cite: 4]
        );

        let uart = XiaoUart::new(
            peripherals.sercom4,
            &mut peripherals.pm,
            &mut peripherals.gclk,
            pins.pb08.into_alternate(), // D6: SERCOM4 TX[cite: 4]
            pins.pb09.into_alternate(), // D7: SERCOM4 RX[cite: 4]
        );

        Self {
            led,
            screen,
            inputs,
            uart,
        }
    }
}

impl PetPlatform for XiaoBoard {
    fn render(&mut self, scene: &RenderScene) {
        self.screen.render(scene);
    }

    fn poll_input(&mut self) -> Option<InputEvent> {
        self.inputs.poll()
    }

    fn is_sleep_switch_on(&self) -> bool {
        self.inputs.is_sleep_switch_on()
    }

    fn read_uart_byte(&mut self) -> Option<u8> {
        self.uart.read_byte()
    }

    fn write_uart_bytes(&mut self, bytes: &[u8]) {
        self.uart.write_bytes(bytes);
    }

    fn toggle_status_led(&mut self) {
        let _ = self.led.toggle();
    }

    fn delay_tick(&mut self) {
        asm::delay(1_000);
    }
}