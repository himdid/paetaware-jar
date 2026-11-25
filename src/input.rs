use atsamd_hal::gpio::{Input, Pin, PullUp, PA02, PA04, PA06, PA10, PA11};
use atsamd_hal::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Left,
    Right,
    Confirm,
    Cancel,
    Step,
}

pub struct Inputs {
    left: Pin<PA02, Input<PullUp>>,
    right: Pin<PA04, Input<PullUp>>,
    confirm: Pin<PA10, Input<PullUp>>,
    cancel: Pin<PA11, Input<PullUp>>,
    step_switch: Pin<PA06, Input<PullUp>>,
    armed: bool,
    release_streak: u8,
}

impl Inputs {
    pub fn new(
        left: Pin<PA02, Input<PullUp>>,
        right: Pin<PA04, Input<PullUp>>,
        confirm: Pin<PA10, Input<PullUp>>,
        cancel: Pin<PA11, Input<PullUp>>,
        step_switch: Pin<PA06, Input<PullUp>>,
    ) -> Self {
        Self {
            left,
            right,
            confirm,
            cancel,
            step_switch,
            armed: true,
            release_streak: 0,
        }
    }

    /// Fires strictly once per press and will not re-arm until all buttons
    /// stay cleanly released for 15 consecutive polls (eliminates bounce).
    pub fn poll(&mut self) -> Option<InputEvent> {
        let current = [
            self.left.is_low().unwrap_or(false),
            self.right.is_low().unwrap_or(false),
            self.confirm.is_low().unwrap_or(false),
            self.cancel.is_low().unwrap_or(false),
            self.step_switch.is_low().unwrap_or(false),
        ];

        let any_pressed = current.iter().any(|&p| p);

        if !self.armed {
            if !any_pressed {
                self.release_streak += 1;
                if self.release_streak >= 15 {
                    self.armed = true;
                    self.release_streak = 0;
                }
            } else {
                self.release_streak = 0;
            }
            return None;
        }

        let map = [
            InputEvent::Left,
            InputEvent::Right,
            InputEvent::Confirm,
            InputEvent::Cancel,
            InputEvent::Step,
        ];

        for i in 0..5 {
            if current[i] {
                self.armed = false;
                self.release_streak = 0;
                return Some(map[i]);
            }
        }

        None
    }
}