use crate::core::face::{FaceEngine, Mood};
use crate::platform::{BubbleRender, EyeRenderState, RenderScene};
use crate::protocol::{CorePayload, DockShell, IncomingPacket};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MenuOption {
    Ping = 0, // PING: Tap containment glass / neural stimulus[cite: 5]
    Dose = 1, // DOSE: Flush nutrient fluid (costs 2 step credits)[cite: 5]
    Scan = 2, // SCAN: Specimen telemetry & wallet readout[cite: 5]
}

impl MenuOption {
    pub fn from_index(idx: u8) -> Self {
        match idx % 3 {
            0 => MenuOption::Ping,
            1 => MenuOption::Dose,
            _ => MenuOption::Scan,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum JarMode {
    Active,
    Stasis,
}

pub struct Pet {
    pub fluid: u8,         // 0..=4 (Nutrient Electrolyte Level)[cite: 5]
    pub lucidity: u8,      // 0..=4 (Neural Clarity / Engagement)[cite: 5]
    pub static_load: u8,   // 0..=4 (Synaptic Noise / Agitation)[cite: 5]
    pub bond: u8,          // 0..=255 (Affinity / Bond)[cite: 5]
    pub steps: u32,        // Lifetime kinetic steps (XP)[cite: 5]
    pub credits: u16,      // Spendable step currency (1 coin per 10 steps)[cite: 5]
    pub mode: JarMode,
    pub docked_shell: DockShell,
    pub selected_menu: u8,
    pub show_stats: bool,
    pub hud_ticks: u8,
    pub status_banner: &'static str,
    pub face: FaceEngine,
    pub bubbles: [BubbleRender; 5],
    pub bubble_boost: u8,
    banner_ticks: u8,
    decay_ticks: u16,
    rng_state: u32,
}

impl Pet {
    pub fn new() -> Self {
        Self {
            fluid: 3,
            lucidity: 3,
            static_load: 1,
            bond: 10,
            steps: 0,
            credits: 5, // Starting reserve of 5 step coins[cite: 5]
            mode: JarMode::Active,
            docked_shell: DockShell::None,
            selected_menu: 0,
            show_stats: false,
            hud_ticks: 0,
            status_banner: "",
            face: FaceEngine::new(),
            bubbles: [
                BubbleRender { x: 14, y: 54, speed: 1, large: false },
                BubbleRender { x: 112, y: 40, speed: 2, large: true },
                BubbleRender { x: 22, y: 24, speed: 1, large: false },
                BubbleRender { x: 105, y: 18, speed: 1, large: false },
                BubbleRender { x: 64, y: 58, speed: 2, large: false },
            ],
            bubble_boost: 0,
            banner_ticks: 0,
            decay_ticks: 0,
            rng_state: 0x5EED_CAFE,
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

    pub fn level(&self) -> u8 {
        if self.steps >= 500 {
            3
        } else if self.steps >= 150 {
            2
        } else {
            1
        }
    }

    pub fn specimen_grade(&self) -> &'static str {
        if self.docked_shell != DockShell::None {
            return self.docked_shell.label();
        }
        match self.level() {
            3 => "CORE-C",
            2 => "CORE-B",
            _ => "CORE-A",
        }
    }

    pub fn spend_credits(&mut self, cost: u16) -> bool {
        if self.credits >= cost {
            self.credits -= cost;
            true
        } else {
            false
        }
    }

    pub fn to_payload(&self) -> CorePayload {
        let xp = (self.steps.min(0xFFFF)) as u16;
        CorePayload {
            species_id: 0x01,
            level: self.level(),
            xp_lo: (xp & 0xFF) as u8,
            xp_hi: (xp >> 8) as u8,
            fluid: self.fluid,
            lucidity: self.lucidity,
            static_load: self.static_load,
            mood: self.face.mood as u8,
            bond: self.bond,
            credits_lo: (self.credits & 0xFF) as u8,
            credits_hi: (self.credits >> 8) as u8,
            reserved: 0x00,
        }
    }

    pub fn to_scene(&self) -> RenderScene<'_> {
        RenderScene {
            show_stats: self.show_stats,
            stasis: self.mode == JarMode::Stasis,
            hud_visible: self.hud_ticks > 0,
            fluid: self.fluid,
            lucidity: self.lucidity,
            static_load: self.static_load,
            steps: self.steps,
            credits: self.credits,
            selected_menu: self.selected_menu,
            status_banner: self.status_banner,
            grade_label: self.specimen_grade(),
            eye: EyeRenderState {
                w: self.face.current.w,
                h: self.face.current.h,
                radius: self.face.current.radius,
                top_lid: self.face.current.top_lid,
                tilt: self.face.current.tilt,
                bottom_arch: self.face.current.bottom_arch,
                look_x: self.face.current.look_x,
                look_y: self.face.current.look_y,
                swirl_phase: if self.face.mood == Mood::Dizzy {
                    Some(self.face.swirl_phase)
                } else {
                    None
                },
                show_cheeks: self.face.show_cheeks,
                show_tear: self.face.show_tear,
            },
            bubbles: &self.bubbles,
        }
    }

    pub fn handle_packet(&mut self, pkt: IncomingPacket) {
        match pkt {
            IncomingPacket::ShellDocked(shell) => {
                self.docked_shell = shell;
                self.show_hud();
                if shell == DockShell::None {
                    self.set_banner("UNDOCK", 25);
                } else {
                    self.face.set_temporary_mood(Mood::Surprised, 35);
                    self.set_banner(shell.label(), 35);
                }
            }
            IncomingPacket::Stimulus { code, intensity } => {
                self.bubble_boost = 15;
                match code {
                    0x01 => {
                        self.lucidity = (self.lucidity + 1).min(4);
                        self.bond = self.bond.saturating_add(intensity.max(1));
                        self.face.set_temporary_mood(Mood::Happy, 35);
                        self.set_banner("SYNC!", 20);
                    }
                    0x02 => {
                        self.static_load = (self.static_load + 1).min(4);
                        self.face.set_temporary_mood(Mood::Dizzy, 40);
                        self.set_banner("JOLT!", 20);
                    }
                    _ => {}
                }
            }
            IncomingPacket::CreditsDelta { spend, amount } => {
                self.show_hud();
                if spend {
                    if self.spend_credits(amount) {
                        self.set_banner("-COIN", 20);
                    } else {
                        self.face.set_temporary_mood(Mood::Lonely, 30);
                        self.set_banner("NO COIN", 20);
                    }
                } else {
                    self.credits = self.credits.saturating_add(amount);
                    self.face.set_temporary_mood(Mood::Happy, 30);
                    self.set_banner("+COIN!", 20);
                }
            }
        }
    }

    fn show_hud(&mut self) {
        self.hud_ticks = 50;
    }

    fn set_banner(&mut self, msg: &'static str, ticks: u8) {
        self.status_banner = msg;
        self.banner_ticks = ticks;
    }

    pub fn move_left(&mut self) {
        if self.mode == JarMode::Stasis {
            return;
        }
        self.show_hud();
        self.selected_menu = (self.selected_menu + 2) % 3;
        self.face.register_poke(Mood::Calm);
    }

    pub fn move_right(&mut self) {
        if self.mode == JarMode::Stasis {
            return;
        }
        self.show_hud();
        self.selected_menu = (self.selected_menu + 1) % 3;
        self.face.register_poke(Mood::Calm);
    }

    pub fn set_sleeping(&mut self, sleeping: bool) -> bool {
        if sleeping && self.mode != JarMode::Stasis {
            self.mode = JarMode::Stasis;
            self.show_stats = false;
            self.hud_ticks = 0;
            self.set_banner("STASIS", 25);
            true
        } else if !sleeping && self.mode == JarMode::Stasis {
            self.mode = JarMode::Active;
            self.set_banner("AWAKE!", 25);
            true
        } else {
            false
        }
    }

    pub fn record_step(&mut self) {
        self.steps = self.steps.saturating_add(1);
        self.face.register_jolt();
        self.bubble_boost = 18;

        // Every 10 steps mints +1 spendable step coin and refreshes lucidity[cite: 5]
        if self.steps % 10 == 0 {
            self.credits = self.credits.saturating_add(1);
            self.lucidity = (self.lucidity + 1).min(4);
            self.static_load = self.static_load.saturating_sub(1);
            if self.mode != JarMode::Stasis && self.face.mood != Mood::Dizzy {
                self.face.set_temporary_mood(Mood::Happy, 35);
                self.set_banner("+1 COIN", 20);
            }
        }
    }

    pub fn confirm(&mut self) {
        if self.mode == JarMode::Stasis {
            return;
        }
        if self.show_stats {
            self.show_stats = false;
            self.show_hud();
            return;
        }

        if self.hud_ticks == 0 {
            self.show_hud();
            self.face.register_poke(Mood::Surprised);
            self.set_banner("*TAP*", 15);
            return;
        }

        self.show_hud();
        match MenuOption::from_index(self.selected_menu) {
            MenuOption::Ping => {
                self.lucidity = (self.lucidity + 1).min(4);
                self.static_load = (self.static_load + 1).min(4);
                self.bond = self.bond.saturating_add(1);
                self.bubble_boost = 12;
                self.face.register_poke(Mood::Happy);
                if self.face.mood == Mood::Angry {
                    self.set_banner("NO TAP!", 25);
                } else {
                    self.set_banner("PING!", 20);
                }
            }
            MenuOption::Dose => {
                // Costs 2 step coins for a nutrient flush (free emergency drip if fluid == 0)[cite: 5]
                if self.fluid == 0 || self.spend_credits(2) {
                    self.fluid = (self.fluid + 1).min(4);
                    self.static_load = self.static_load.saturating_sub(1);
                    self.bond = self.bond.saturating_add(2);
                    self.bubble_boost = 22;
                    self.face.set_temporary_mood(Mood::Happy, 35);
                    self.set_banner("DOSE-2C", 20);
                } else {
                    self.face.set_temporary_mood(Mood::Lonely, 30);
                    self.set_banner("NEED 2C", 20);
                }
            }
            MenuOption::Scan => {
                self.show_stats = true;
            }
        }
    }

    pub fn cancel(&mut self) {
        if self.show_stats {
            self.show_stats = false;
        }
        self.hud_ticks = 0;
    }

    fn update_bubbles(&mut self) {
        if self.bubble_boost > 0 {
            self.bubble_boost -= 1;
        }
        let extra = if self.bubble_boost > 0 { 1 } else { 0 };
        let stasis = self.mode == JarMode::Stasis;

        for i in 0..self.bubbles.len() {
            let step = if stasis {
                1
            } else {
                self.bubbles[i].speed + extra
            };
            self.bubbles[i].y -= step;

            if (self.bubbles[i].y & 0x04) == 0 {
                self.bubbles[i].x += if (i & 1) == 0 { 1 } else { -1 };
            }

            if self.bubbles[i].y < 4 {
                let r = self.next_rand();
                self.bubbles[i].y = 60;
                self.bubbles[i].x = if (r & 1) == 0 {
                    6 + ((r >> 4) % 18) as i16
                } else {
                    104 + ((r >> 4) % 18) as i16
                };
                self.bubbles[i].large = ((r >> 9) % 4) == 0;
            }
        }
    }

    pub fn tick(&mut self) {
        self.face.update(self.mode == JarMode::Stasis);
        self.update_bubbles();

        if self.hud_ticks > 0 {
            self.hud_ticks -= 1;
        }

        if self.banner_ticks > 0 {
            self.banner_ticks -= 1;
            if self.banner_ticks == 0 {
                self.status_banner = "";
            }
        }

        self.decay_ticks += 1;
        if self.decay_ticks >= 160 {
            self.decay_ticks = 0;
            if self.mode == JarMode::Stasis {
                self.static_load = self.static_load.saturating_sub(1);
            } else {
                self.fluid = self.fluid.saturating_sub(1);
                self.lucidity = self.lucidity.saturating_sub(1);
            }
        }
    }
}