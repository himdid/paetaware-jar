use atsamd_hal::gpio::{Pin, PushPullOutput, PA08, PA09};
use atsamd_hal::prelude::*;
use cortex_m::asm;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_hal::blocking::i2c::Write;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::sprites::SPRITE_WIDTH;

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
        asm::delay(250);
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

    pub fn clear(&mut self) {
        self.oled.clear(BinaryColor::Off).unwrap();
    }

    /// Draws a 1-bit sprite scaled 2x (each source pixel becomes a 2x2 block)
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

    pub fn flush(&mut self) {
        self.oled.flush().unwrap();
    }
}