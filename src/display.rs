use atsamd_hal::gpio::{Pin, PushPullOutput, PA08, PA09};
use atsamd_hal::prelude::*;
use cortex_m::asm;
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_hal::blocking::i2c::Write;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::pet::{Pet, PetState};
use crate::sprites::{PET_IDLE_0, PET_IDLE_1, SPRITE_WIDTH};

/// Software I2C driver using PA08 (SDA) and PA09 (SCL)
pub struct SoftI2c {
    sda: Pin<PA08, PushPullOutput>,
    scl: Pin<PA09, PushPullOutput>,
}

impl SoftI2c {
    pub fn new(mut sda: Pin<PA08, PushPullOutput>, mut scl: Pin<PA09, PushPullOutput>) -> Self {
        let _ = sda.set_high();
        let _ = scl.set_high();
        Self { sda, scl }
    }

    #[inline(always)]
    fn delay(&self) {
        asm::delay(20);
    }

    fn start(&mut self) {
        let _ = self.sda.set_high();
        let _ = self.scl.set_high();
        self.delay();
        let _ = self.sda.set_low();
        self.delay();
        let _ = self.scl.set_low();
        self.delay();
    }

    fn stop(&mut self) {
        let _ = self.sda.set_low();
        self.delay();
        let _ = self.scl.set_high();
        self.delay();
        let _ = self.sda.set_high();
        self.delay();
    }

    fn write_byte(&mut self, mut byte: u8) {
        for _ in 0..8 {
            if (byte & 0x80) != 0 {
                let _ = self.sda.set_high();
            } else {
                let _ = self.sda.set_low();
            }
            self.delay();
            let _ = self.scl.set_high();
            self.delay();
            let _ = self.scl.set_low();
            self.delay();
            byte <<= 1;
        }
        // 9th clock pulse for ACK
        let _ = self.sda.set_high();
        self.delay();
        let _ = self.scl.set_high();
        self.delay();
        let _ = self.scl.set_low();
        self.delay();
    }
}

impl Write for SoftI2c {
    type Error = ();

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.start();
        self.write_byte(addr << 1);
        for &b in bytes {
            self.write_byte(b);
        }
        self.stop();
        Ok(())
    }
}

type OledDriver = Ssd1306<
    I2CInterface<SoftI2c>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub struct Screen {
    oled: OledDriver,
}

impl Screen {
    pub fn new(sda: Pin<PA08, PushPullOutput>, scl: Pin<PA09, PushPullOutput>) -> Self {
        let i2c = SoftI2c::new(sda, scl);
        let interface = I2CDisplayInterface::new(i2c);
        let mut oled = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
            .into_buffered_graphics_mode();

        oled.init().unwrap();
        oled.clear(BinaryColor::Off).unwrap();
        oled.flush().unwrap();

        Self { oled }
    }

    pub fn draw_sprite_2x(&mut self, data: &[u8], x: i32, y: i32) {
        let on_style = PrimitiveStyle::with_fill(BinaryColor::On);
        let bytes_per_row = (SPRITE_WIDTH / 8) as usize;
        let rows = data.len() / bytes_per_row;

        for row in 0..rows {
            for col in 0..(SPRITE_WIDTH as usize) {
                let byte = data[row * bytes_per_row + (col / 8)];
                let bit_mask = 0x80 >> (col % 8);
                if (byte & bit_mask) != 0 {
                    Rectangle::new(
                        Point::new(x + (col as i32) * 2, y + (row as i32) * 2),
                        Size::new(2, 2),
                    )
                    .into_styled(on_style)
                    .draw(&mut self.oled)
                    .unwrap();
                }
            }
        }
    }

    fn draw_stat_bar(&mut self, label: &str, value: u8, y: i32) {
        let text_style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
        let on_style = PrimitiveStyle::with_fill(BinaryColor::On);
        let outline_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        Text::new(label, Point::new(4, y + 7), text_style)
            .draw(&mut self.oled)
            .unwrap();

        for i in 0..4 {
            let box_x = 44 + (i as i32) * 18;
            let rect = Rectangle::new(Point::new(box_x, y), Size::new(14, 8));
            if i < value {
                rect.into_styled(on_style).draw(&mut self.oled).unwrap();
            } else {
                rect.into_styled(outline_style).draw(&mut self.oled).unwrap();
            }
        }
    }

    pub fn render(&mut self, pet: &Pet, frame: u8) {
        self.oled.clear(BinaryColor::Off).unwrap();

        let text_on = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
        let text_off = MonoTextStyle::new(&FONT_5X8, BinaryColor::Off);
        let fill_on = PrimitiveStyle::with_fill(BinaryColor::On);

        if pet.show_stats {
            Text::new("== PET STATS ==", Point::new(24, 9), text_on)
                .draw(&mut self.oled)
                .unwrap();
            self.draw_stat_bar("HUNGER", pet.hunger, 16);
            self.draw_stat_bar("HAPPY ", pet.happiness, 29);
            self.draw_stat_bar("ENERGY", pet.energy, 42);
            Text::new("[B] BACK", Point::new(42, 61), text_on)
                .draw(&mut self.oled)
                .unwrap();
            self.oled.flush().unwrap();
            return;
        }

        // Top HUD: Step counter (no_std integer formatting without heap allocation)
        let mut step_buf = [b'S', b'T', b'P', b':', b'0', b'0', b'0', b'0'];
        let mut n = pet.steps.min(9999);
        for i in (4..8).rev() {
            step_buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        let step_str = core::str::from_utf8(&step_buf).unwrap_or("STP:0000");
        Text::new(step_str, Point::new(2, 8), text_on)
            .draw(&mut self.oled)
            .unwrap();

        // Top-right state indicator
        let status = match pet.state {
            PetState::Idle => "",
            PetState::Eating => "YUM!",
            PetState::Happy => "<3",
            PetState::Sleeping => "ZZZ",
        };
        if !status.is_empty() {
            Text::new(status, Point::new(102, 8), text_on)
                .draw(&mut self.oled)
                .unwrap();
        }

        // Center 32x32 sprite
        let sprite = match pet.state {
            PetState::Sleeping => &PET_IDLE_1,
            _ => {
                if frame == 0 {
                    &PET_IDLE_0
                } else {
                    &PET_IDLE_1
                }
            }
        };
        self.draw_sprite_2x(sprite, 48, 14);

        // Bottom 4-item menu bar
        let labels = ["FED", "PLY", "SLP", "STS"];
        for (i, label) in labels.iter().enumerate() {
            let x = (i as i32) * 32 + 4;
            if pet.selected_menu == i as u8 {
                Rectangle::new(Point::new(x - 2, 52), Size::new(26, 11))
                    .into_styled(fill_on)
                    .draw(&mut self.oled)
                    .unwrap();
                Text::new(label, Point::new(x + 3, 60), text_off)
                    .draw(&mut self.oled)
                    .unwrap();
            } else {
                Text::new(label, Point::new(x + 3, 60), text_on)
                    .draw(&mut self.oled)
                    .unwrap();
            }
        }

        self.oled.flush().unwrap();
    }
}