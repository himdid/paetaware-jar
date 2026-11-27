pub mod echo;
pub mod face;
pub mod pet;

use crate::platform::PetPlatform;
use crate::protocol::PacketFramer;
use pet::Pet;

pub struct PetEngine {
    pub pet: Pet,
    framer: PacketFramer,
    anim_counter: u16,
    tx_counter: u8,
}

impl PetEngine {
    pub fn new() -> Self {
        Self {
            pet: Pet::new(),
            framer: PacketFramer::new(),
            anim_counter: 0,
            tx_counter: 0,
        }
    }

    pub fn boot(&mut self, hw: &mut impl PetPlatform) {
        self.pet.set_sleeping(hw.is_sleep_switch_on());
        self.send_telemetry(hw);
        hw.render(&self.pet.to_scene());
    }

    pub fn update(&mut self, hw: &mut impl PetPlatform) {
        while let Some(byte) = hw.read_uart_byte() {
            if let Some(pkt) = self.framer.push_byte(byte) {
                self.pet.handle_packet(pkt);
                hw.render(&self.pet.to_scene());
            }
        }

        if self.pet.set_sleeping(hw.is_sleep_switch_on()) {
            self.send_telemetry(hw);
            hw.render(&self.pet.to_scene());
        }

        if let Some(event) = hw.poll_input() {
            self.pet.handle_input(event);
            self.send_telemetry(hw);
            hw.render(&self.pet.to_scene());
        }

        self.anim_counter += 1;
        if self.anim_counter >= 180 {
            self.anim_counter = 0;
            self.pet.tick();
            hw.render(&self.pet.to_scene());

            self.tx_counter += 1;
            if self.tx_counter >= 20 {
                self.tx_counter = 0;
                self.send_telemetry(hw);
                hw.toggle_status_led();
            }
        }

        hw.delay_tick();
    }

    fn send_telemetry(&self, hw: &mut impl PetPlatform) {
        let frame = PacketFramer::encode_telemetry(&self.pet.to_payload());
        hw.write_uart_bytes(&frame);
    }
}