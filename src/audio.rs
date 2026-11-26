use atsamd_hal::gpio::{Pin, PushPullOutput, PA07};
use atsamd_hal::prelude::*;
use cortex_m::asm;

pub struct Buzzer {
    pin: Pin<PA07, PushPullOutput>,
    pub muted: bool,
}

impl Buzzer {
    pub fn new(mut pin: Pin<PA07, PushPullOutput>) -> Self {
        let _ = pin.set_low();
        Self { pin, muted: false }
    }

    /// Plays a square-wave tone at `freq_hz` for `duration_ms`
    pub fn tone(&mut self, freq_hz: u32, duration_ms: u32) {
        if self.muted || freq_hz == 0 || duration_ms == 0 {
            return;
        }

        // Default SAMD21 startup clock is ~1 MHz (or 48 MHz once GCLK0 is configured).
        // Tuned for crisp audible square waves in both Mokxi and physical piezo elements.
        let half_period_cycles = 500_000 / freq_hz;
        let total_cycles = (freq_hz * duration_ms) / 1_000;

        for _ in 0..total_cycles {
            let _ = self.pin.set_high();
            asm::delay(half_period_cycles);
            let _ = self.pin.set_low();
            asm::delay(half_period_cycles);
        }
    }

    /// Short UI cursor blip
    pub fn beep_move(&mut self) {
        self.tone(880, 18);
    }

    /// Positive two-note chime for Confirm / Feed / Play
    pub fn beep_confirm(&mut self) {
        self.tone(988, 25);
        self.tone(1319, 40);
    }

    /// Lower two-note drop for Cancel / Back
    pub fn beep_cancel(&mut self) {
        self.tone(659, 25);
        self.tone(440, 35);
    }

    /// Quick soft tick for a single pedometer step
    pub fn beep_step(&mut self) {
        self.tone(1568, 10);
    }

    /// Three-note victory/happy arpeggio every 10 steps
    pub fn beep_milestone(&mut self) {
        self.tone(1047, 30);
        self.tone(1319, 30);
        self.tone(1568, 55);
    }
}