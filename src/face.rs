#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mood {
    Calm,
    Happy,
    Angry,
    Dizzy,
    Sleepy,
    Asleep,
    Surprised,
    Lonely,
}

#[derive(Copy, Clone)]
pub struct EyePose {
    pub w: i16,
    pub h: i16,
    pub radius: i16,
    pub top_lid: i16,
    pub tilt: i16,
    pub bottom_arch: i16,
    pub look_x: i16,
    pub look_y: i16,
}

impl EyePose {
    pub const fn for_mood(mood: Mood) -> Self {
        match mood {
            Mood::Calm => Self {
                w: 28,
                h: 32,
                radius: 9,
                top_lid: 0,
                tilt: 0,
                bottom_arch: 0,
                look_x: 0,
                look_y: 0,
            },
            Mood::Happy => Self {
                w: 30,
                h: 28,
                radius: 10,
                top_lid: 0,
                tilt: 0,
                bottom_arch: 13,
                look_x: 0,
                look_y: -2,
            },
            Mood::Angry => Self {
                w: 28,
                h: 28,
                radius: 6,
                top_lid: 7,
                tilt: 10,
                bottom_arch: 0,
                look_x: 0,
                look_y: 0,
            },
            Mood::Dizzy => Self {
                w: 18,
                h: 18,
                radius: 9,
                top_lid: 0,
                tilt: 0,
                bottom_arch: 0,
                look_x: 0,
                look_y: 0,
            },
            Mood::Sleepy => Self {
                w: 28,
                h: 24,
                radius: 7,
                top_lid: 14,
                tilt: -3,
                bottom_arch: 0,
                look_x: 0,
                look_y: 3,
            },
            Mood::Asleep => Self {
                w: 30,
                h: 4,
                radius: 2,
                top_lid: 0,
                tilt: 0,
                bottom_arch: 0,
                look_x: 0,
                look_y: 4,
            },
            Mood::Surprised => Self {
                w: 32,
                h: 34,
                radius: 11,
                top_lid: 0,
                tilt: 0,
                bottom_arch: 0,
                look_x: 0,
                look_y: -1,
            },
            Mood::Lonely => Self {
                w: 24,
                h: 24,
                radius: 7,
                top_lid: 5,
                tilt: -7,
                bottom_arch: 0,
                look_x: -3,
                look_y: 5,
            },
        }
    }

    fn step_toward(cur: i16, target: i16) -> i16 {
        let diff = target - cur;
        if diff.abs() <= 1 {
            target
        } else {
            cur + (diff / 2)
        }
    }

    pub fn interpolate(&mut self, target: &EyePose) {
        self.w = Self::step_toward(self.w, target.w);
        self.h = Self::step_toward(self.h, target.h);
        self.radius = Self::step_toward(self.radius, target.radius);
        self.top_lid = Self::step_toward(self.top_lid, target.top_lid);
        self.tilt = Self::step_toward(self.tilt, target.tilt);
        self.bottom_arch = Self::step_toward(self.bottom_arch, target.bottom_arch);
        self.look_x = Self::step_toward(self.look_x, target.look_x);
        self.look_y = Self::step_toward(self.look_y, target.look_y);
    }
}

pub struct FaceEngine {
    pub mood: Mood,
    pub current: EyePose,
    pub swirl_phase: u8,
    pub show_cheeks: bool,
    pub show_tear: bool,
    override_ticks: u16,
    blink_timer: u16,
    blink_frames: u8,
    glance_timer: u16,
    glance_x: i16,
    glance_y: i16,
    idle_ticks: u16,
    poke_streak: u8,
    poke_window: u16,
    wobble_score: u8,
    sleep_ticks: u16,
    rng_state: u32,
}

impl FaceEngine {
    pub fn new() -> Self {
        Self {
            mood: Mood::Surprised,
            current: EyePose::for_mood(Mood::Surprised),
            swirl_phase: 0,
            show_cheeks: false,
            show_tear: false,
            override_ticks: 24,
            blink_timer: 50,
            blink_frames: 0,
            glance_timer: 35,
            glance_x: 0,
            glance_y: 0,
            idle_ticks: 0,
            poke_streak: 0,
            poke_window: 0,
            wobble_score: 0,
            sleep_ticks: 0,
            rng_state: 0xA5A5_1234,
        }
    }

    fn next_rand(&mut self) -> u32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        x
    }

    pub fn set_temporary_mood(&mut self, mood: Mood, ticks: u16) {
        self.mood = mood;
        self.override_ticks = ticks;
        self.idle_ticks = 0;
    }

    pub fn register_poke(&mut self, positive_mood: Mood) {
        self.idle_ticks = 0;
        if self.poke_window > 0 {
            self.poke_streak = self.poke_streak.saturating_add(1);
        } else {
            self.poke_streak = 1;
        }
        self.poke_window = 28;

        if self.poke_streak >= 5 {
            self.set_temporary_mood(Mood::Angry, 60);
        } else if self.mood != Mood::Angry {
            self.set_temporary_mood(positive_mood, 35);
        }
    }

    pub fn register_jolt(&mut self) {
        self.idle_ticks = 0;
        self.wobble_score = self.wobble_score.saturating_add(3);
        if self.wobble_score >= 12 {
            self.set_temporary_mood(Mood::Dizzy, 55);
            self.wobble_score = 0;
        }
    }

    pub fn update(&mut self, sleeping: bool) {
        if self.poke_window > 0 {
            self.poke_window -= 1;
            if self.poke_window == 0 {
                self.poke_streak = 0;
            }
        }

        if self.wobble_score > 0 {
            self.wobble_score -= 1;
        }

        if sleeping {
            self.sleep_ticks = self.sleep_ticks.saturating_add(1);
            self.override_ticks = 0;
            self.mood = if self.sleep_ticks < 45 {
                Mood::Sleepy
            } else {
                Mood::Asleep
            };
        } else {
            if self.sleep_ticks > 0 {
                self.sleep_ticks = 0;
                self.set_temporary_mood(Mood::Surprised, 22);
            }

            if self.override_ticks > 0 {
                self.override_ticks -= 1;
            } else {
                self.idle_ticks = self.idle_ticks.saturating_add(1);
                if self.idle_ticks >= 600 {
                    self.mood = Mood::Lonely;
                } else {
                    self.mood = Mood::Calm;
                }
            }
        }

        let mut target = EyePose::for_mood(self.mood);

        if matches!(self.mood, Mood::Calm | Mood::Lonely | Mood::Sleepy) {
            if self.glance_timer > 0 {
                self.glance_timer -= 1;
            } else {
                let r = self.next_rand();
                self.glance_timer = 30 + (r % 45) as u16;
                self.glance_x = match (r >> 8) % 5 {
                    0 => -7,
                    1 => 7,
                    _ => 0,
                };
                self.glance_y = match (r >> 12) % 4 {
                    0 => -3,
                    1 => 3,
                    _ => 0,
                };
            }
            target.look_x += self.glance_x;
            target.look_y += self.glance_y;
        }

        if self.mood == Mood::Dizzy {
            self.swirl_phase = (self.swirl_phase + 1) % 8;
        }

        if self.mood != Mood::Asleep && self.mood != Mood::Dizzy {
            if self.blink_frames > 0 {
                self.blink_frames -= 1;
                target.h = 3;
                target.top_lid = 0;
                target.bottom_arch = 0;
            } else if self.blink_timer > 0 {
                self.blink_timer -= 1;
            } else {
                self.blink_frames = 3;
                let fast_blink = if self.mood == Mood::Sleepy { 25 } else { 55 };
                self.blink_timer = fast_blink + (self.next_rand() % 50) as u16;
            }
        }

        self.show_cheeks = self.mood == Mood::Happy;
        self.show_tear = self.mood == Mood::Lonely && ((self.idle_ticks / 12) % 3 == 0);

        self.current.interpolate(&target);
    }
}