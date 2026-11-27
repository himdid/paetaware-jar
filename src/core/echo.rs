use crate::platform::{EchoHud, InputEvent};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EchoDir {
    Left = 0,
    Right = 1,
    Up = 2,   // Confirm button (D2)
    Down = 3, // Cancel button (D3)
}

impl EchoDir {
    pub fn from_u8(v: u8) -> Self {
        match v % 4 {
            0 => EchoDir::Left,
            1 => EchoDir::Right,
            2 => EchoDir::Up,
            _ => EchoDir::Down,
        }
    }

    pub fn gaze_offset(self) -> (i16, i16) {
        match self {
            EchoDir::Left => (-11, 0),
            EchoDir::Right => (11, 0),
            EchoDir::Up => (0, 8),
            EchoDir::Down => (0, -8),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EchoDir::Left => "LEFT",
            EchoDir::Right => "RIGHT",
            EchoDir::Up => "DN [B]",
            EchoDir::Down => "UP [A]",
        }
    }
}

pub enum EchoEvent {
    Continue {
        gaze: Option<(i16, i16)>,
        banner: Option<(&'static str, u8)>,
        bubble_boost: bool,
    },
    CashedOut {
        pot: u16,
    },
    Lost,
}

const MAX_ECHO_STEPS: usize = 16;

pub struct EchoGame {
    seq: [EchoDir; MAX_ECHO_STEPS],
    len: u8,
    round: u8,
    pot: u16,
    brain_turn: bool,
    step_idx: u8,
    phase_ticks: u8,
    showing_dir: bool,
}

impl EchoGame {
    pub fn new(first_dir: EchoDir) -> Self {
        let mut seq = [EchoDir::Left; MAX_ECHO_STEPS];
        seq[0] = first_dir;
        Self {
            seq,
            len: 1,
            round: 1,
            pot: 0,
            brain_turn: true,
            step_idx: 0,
            phase_ticks: 10,
            showing_dir: false,
        }
    }

    pub fn to_hud(&self) -> EchoHud {
        EchoHud {
            round: self.round,
            pot: self.pot,
            brain_turn: self.brain_turn,
        }
    }

    pub fn handle_input(&mut self, event: InputEvent, next_rand_byte: u8) -> EchoEvent {
        if self.brain_turn {
            if event == InputEvent::Cancel {
                return EchoEvent::CashedOut { pot: self.pot };
            }
            return EchoEvent::Continue {
                gaze: None,
                banner: None,
                bubble_boost: false,
            };
        }

        let guess = match event {
            InputEvent::Left => EchoDir::Left,
            InputEvent::Right => EchoDir::Right,
            InputEvent::Confirm => EchoDir::Up,
            InputEvent::Cancel => EchoDir::Down,
            InputEvent::Step => {
                return EchoEvent::Continue {
                    gaze: None,
                    banner: None,
                    bubble_boost: false,
                }
            }
        };

        let expected = self.seq[self.step_idx as usize];
        if guess != expected {
            return EchoEvent::Lost;
        }

        self.step_idx += 1;
        if self.step_idx >= self.len {
            // Round cleared: add round number to pot and append next random step
            self.pot = self.pot.saturating_add(self.round as u16);
            if (self.len as usize) < MAX_ECHO_STEPS {
                self.seq[self.len as usize] = EchoDir::from_u8(next_rand_byte);
                self.len += 1;
            }
            self.round = self.round.saturating_add(1);
            self.brain_turn = true;
            self.step_idx = 0;
            self.showing_dir = false;
            self.phase_ticks = 12;

            EchoEvent::Continue {
                gaze: Some((0, 0)),
                banner: Some(("NEXT!", 15)),
                bubble_boost: true,
            }
        } else {
            EchoEvent::Continue {
                gaze: Some(guess.gaze_offset()),
                banner: None,
                bubble_boost: false,
            }
        }
    }

    pub fn tick(&mut self) -> (Option<(i16, i16)>, Option<(&'static str, u8)>) {
        if !self.brain_turn {
            return (None, None);
        }

        if self.phase_ticks > 0 {
            self.phase_ticks -= 1;
            return (None, None);
        }

        if self.showing_dir {
            self.showing_dir = false;
            self.step_idx += 1;
            self.phase_ticks = 4;
            (Some((0, 0)), None)
        } else if self.step_idx < self.len {
            let dir = self.seq[self.step_idx as usize];
            self.showing_dir = true;
            self.phase_ticks = 8;
            (Some(dir.gaze_offset()), Some((dir.label(), 8)))
        } else {
            self.brain_turn = false;
            self.step_idx = 0;
            (Some((0, 0)), Some(("YOUR TURN", 15)))
        }
    }
}