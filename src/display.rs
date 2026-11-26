use atsamd_hal::gpio::{Pin, PushPullOutput, PA08, PA09};
use atsamd_hal::prelude::*;
use cortex_m::asm;
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle, Triangle},
    text::Text,
};
use embedded_hal::blocking::i2c::Write;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::face::Mood;
use crate::pet::{JarMode, Pet};

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

    fn draw_jar_atmosphere(&mut self, pet: &Pet, hud_visible: bool) {
        if pet.mode == JarMode::Stasis {
            return;
        }

        let on_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let on_fill = PrimitiveStyle::with_fill(BinaryColor::On);

        for b in &pet.bubbles {
            if hud_visible && (b.y < 12 || b.y > 49) {
                continue;
            }
            if b.large {
                Circle::new(Point::new(b.x as i32, b.y as i32), 4)
                    .into_styled(on_stroke)
                    .draw(&mut self.oled)
                    .unwrap();
            } else {
                Rectangle::new(Point::new(b.x as i32, b.y as i32), Size::new(2, 2))
                    .into_styled(on_fill)
                    .draw(&mut self.oled)
                    .unwrap();
            }
        }
    }

    fn draw_procedural_face(&mut self, pet: &Pet, base_y: i32) {
        let on = PrimitiveStyle::with_fill(BinaryColor::On);
        let off = PrimitiveStyle::with_fill(BinaryColor::Off);
        let pose = &pet.face.current;

        let w = pose.w.max(4) as i32;
        let h = pose.h.max(3) as i32;
        let r = (pose.radius.max(0) as u32).min((w as u32) / 2).min((h as u32) / 2);

        let (swirl_lx, swirl_ly, swirl_rx, swirl_ry) = if pet.face.mood == Mood::Dizzy {
            const DX: [i32; 8] = [0, 4, 5, 4, 0, -4, -5, -4];
            const DY: [i32; 8] = [-4, -3, 0, 3, 4, 3, 0, -3];
            let i = (pet.face.swirl_phase as usize) % 8;
            let j = (i + 4) % 8;
            (DX[i], DY[i], DX[j], DY[j])
        } else {
            (0, 0, 0, 0)
        };

        let left_cx = 36 + (pose.look_x as i32) + swirl_lx;
        let right_cx = 92 + (pose.look_x as i32) + swirl_rx;
        let left_cy = base_y + (pose.look_y as i32) + swirl_ly;
        let right_cy = base_y + (pose.look_y as i32) + swirl_ry;

        for (is_left, (cx, cy)) in [(true, (left_cx, left_cy)), (false, (right_cx, right_cy))] {
            let x0 = cx - (w / 2);
            let y0 = cy - (h / 2);

            RoundedRectangle::new(
                Rectangle::new(Point::new(x0, y0), Size::new(w as u32, h as u32)),
                CornerRadii::new(Size::new(r, r)),
            )
            .into_styled(on)
            .draw(&mut self.oled)
            .unwrap();

            let top_drop = (pose.top_lid.max(0) as i32).min(h - 2);
            if top_drop > 0 {
                Rectangle::new(
                    Point::new(x0 - 1, y0 - 1),
                    Size::new((w + 2) as u32, (top_drop + 1) as u32),
                )
                .into_styled(off)
                .draw(&mut self.oled)
                .unwrap();
            }

            let tilt = pose.tilt as i32;
            let lid_y = y0 + top_drop;
            if tilt > 0 {
                if is_left {
                    Triangle::new(
                        Point::new(x0 - 1, lid_y - 1),
                        Point::new(x0 + w + 1, lid_y - 1),
                        Point::new(x0 + w + 1, lid_y + tilt),
                    )
                    .into_styled(off)
                    .draw(&mut self.oled)
                    .unwrap();
                } else {
                    Triangle::new(
                        Point::new(x0 - 1, lid_y - 1),
                        Point::new(x0 + w + 1, lid_y - 1),
                        Point::new(x0 - 1, lid_y + tilt),
                    )
                    .into_styled(off)
                    .draw(&mut self.oled)
                    .unwrap();
                }
            } else if tilt < 0 {
                let sad = -tilt;
                if is_left {
                    Triangle::new(
                        Point::new(x0 - 1, lid_y - 1),
                        Point::new(x0 + w + 1, lid_y - 1),
                        Point::new(x0 - 1, lid_y + sad),
                    )
                    .into_styled(off)
                    .draw(&mut self.oled)
                    .unwrap();
                } else {
                    Triangle::new(
                        Point::new(x0 - 1, lid_y - 1),
                        Point::new(x0 + w + 1, lid_y - 1),
                        Point::new(x0 + w + 1, lid_y + sad),
                    )
                    .into_styled(off)
                    .draw(&mut self.oled)
                    .unwrap();
                }
            }

            let arch = (pose.bottom_arch.max(0) as i32).min(h - 3);
            if arch > 0 {
                Circle::new(
                    Point::new(x0 + 2, y0 + h - arch),
                    (w - 4).max(4) as u32,
                )
                .into_styled(off)
                .draw(&mut self.oled)
                .unwrap();
            }
        }

        if pet.face.show_cheeks {
            Rectangle::new(Point::new(left_cx - 17, left_cy + 13), Size::new(7, 2))
                .into_styled(on)
                .draw(&mut self.oled)
                .unwrap();
            Rectangle::new(Point::new(right_cx + 10, right_cy + 13), Size::new(7, 2))
                .into_styled(on)
                .draw(&mut self.oled)
                .unwrap();
        }

        if pet.face.show_tear {
            Rectangle::new(Point::new(left_cx - 10, left_cy + 14), Size::new(2, 5))
                .into_styled(on)
                .draw(&mut self.oled)
                .unwrap();
        }
    }

    pub fn render(&mut self, pet: &Pet) {
        self.oled.clear(BinaryColor::Off).unwrap();

        let text_on = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
        let text_off = MonoTextStyle::new(&FONT_5X8, BinaryColor::Off);
        let fill_on = PrimitiveStyle::with_fill(BinaryColor::On);

        // Stats screen (SCAN): displays vitals + lifetime STEP count
        if pet.show_stats {
            Text::new("== SPECIMEN SCAN ==", Point::new(15, 8), text_on)
                .draw(&mut self.oled)
                .unwrap();
            self.draw_stat_bar("FLUID ", pet.fluid, 14);
            self.draw_stat_bar("LUCID ", pet.lucidity, 26);
            self.draw_stat_bar("STATIC", pet.static_load, 38);

            let mut stp_buf = [
                b'S', b'T', b'E', b'P', b'S', b':', b'0', b'0', b'0', b'0', b' ', b'[', b'B', b']',
            ];
            let mut s = pet.steps.min(9999);
            for i in (6..10).rev() {
                stp_buf[i] = b'0' + (s % 10) as u8;
                s /= 10;
            }
            let stp_str = core::str::from_utf8(&stp_buf).unwrap_or("STEPS:0000 [B]");
            Text::new(stp_str, Point::new(28, 60), text_on)
                .draw(&mut self.oled)
                .unwrap();

            self.oled.flush().unwrap();
            return;
        }

        let hud_visible = pet.hud_ticks > 0;
        let face_y = if hud_visible { 28 } else { 32 };

        self.draw_jar_atmosphere(pet, hud_visible);
        self.draw_procedural_face(pet, face_y);

        if !pet.status_banner.is_empty() {
            Text::new(pet.status_banner, Point::new(84, 8), text_on)
                .draw(&mut self.oled)
                .unwrap();
        }

        // Main screen HUD: displays spendable COIN balance top-left
        if hud_visible {
            let mut coin_buf = [b'C', b'O', b'I', b'N', b':', b'0', b'0', b'0', b'0'];
            let mut c = pet.credits.min(9999);
            for i in (5..9).rev() {
                coin_buf[i] = b'0' + (c % 10) as u8;
                c /= 10;
            }
            let coin_str = core::str::from_utf8(&coin_buf).unwrap_or("COIN:0000");
            Text::new(coin_str, Point::new(2, 8), text_on)
                .draw(&mut self.oled)
                .unwrap();

            if pet.status_banner.is_empty() {
                Text::new(pet.specimen_grade(), Point::new(92, 8), text_on)
                    .draw(&mut self.oled)
                    .unwrap();
            }

            let labels = ["PING", "DOSE", "SCAN"];
            for (i, label) in labels.iter().enumerate() {
                let x = (i as i32) * 42 + 8;
                if pet.selected_menu == i as u8 {
                    Rectangle::new(Point::new(x - 2, 52), Size::new(28, 11))
                        .into_styled(fill_on)
                        .draw(&mut self.oled)
                        .unwrap();
                    Text::new(label, Point::new(x + 2, 60), text_off)
                        .draw(&mut self.oled)
                        .unwrap();
                } else {
                    Text::new(label, Point::new(x + 2, 60), text_on)
                        .draw(&mut self.oled)
                        .unwrap();
                }
            }
        }

        self.oled.flush().unwrap();
    }
}