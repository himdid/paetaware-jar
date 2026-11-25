#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MenuOption {
    Feed = 0,
    Play = 1,
    Sleep = 2,
    Stats = 3,
}

impl MenuOption {
    pub fn from_index(idx: u8) -> Self {
        match idx % 4 {
            0 => MenuOption::Feed,
            1 => MenuOption::Play,
            2 => MenuOption::Sleep,
            _ => MenuOption::Stats,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PetState {
    Idle,
    Eating,
    Happy,
    Sleeping,
}

pub struct Pet {
    pub hunger: u8,     // 0..=4
    pub happiness: u8,  // 0..=4
    pub energy: u8,     // 0..=4
    pub steps: u32,
    pub state: PetState,
    pub selected_menu: u8,
    pub show_stats: bool,
    state_timer: u8,
    decay_ticks: u16,
}

impl Pet {
    pub fn new() -> Self {
        Self {
            hunger: 3,
            happiness: 3,
            energy: 4,
            steps: 0,
            state: PetState::Idle,
            selected_menu: 0,
            show_stats: false,
            state_timer: 0,
            decay_ticks: 0,
        }
    }

    pub fn move_left(&mut self) {
        self.selected_menu = (self.selected_menu + 3) % 4;
    }

    pub fn move_right(&mut self) {
        self.selected_menu = (self.selected_menu + 1) % 4;
    }

    pub fn record_step(&mut self) {
        self.steps = self.steps.saturating_add(1);
        // Every 10 steps boosts happiness by 1 (and wakes pet if sleeping)
        if self.steps % 10 == 0 {
            self.happiness = (self.happiness + 1).min(4);
            if self.state != PetState::Sleeping {
                self.state = PetState::Happy;
                self.state_timer = 3;
            }
        }
    }

    pub fn confirm(&mut self) {
        if self.show_stats {
            self.show_stats = false;
            return;
        }

        match MenuOption::from_index(self.selected_menu) {
            MenuOption::Feed => {
                if self.state != PetState::Sleeping {
                    self.hunger = (self.hunger + 1).min(4);
                    self.state = PetState::Eating;
                    self.state_timer = 3;
                }
            }
            MenuOption::Play => {
                if self.state != PetState::Sleeping && self.energy > 0 {
                    self.happiness = (self.happiness + 1).min(4);
                    self.energy = self.energy.saturating_sub(1);
                    self.state = PetState::Happy;
                    self.state_timer = 3;
                }
            }
            MenuOption::Sleep => {
                self.state = if self.state == PetState::Sleeping {
                    PetState::Idle
                } else {
                    PetState::Sleeping
                };
            }
            MenuOption::Stats => {
                self.show_stats = true;
            }
        }
    }

    pub fn cancel(&mut self) {
        if self.show_stats {
            self.show_stats = false;
        } else if self.state == PetState::Sleeping {
            self.state = PetState::Idle;
        }
    }

    pub fn tick(&mut self) {
        if self.state_timer > 0 {
            self.state_timer -= 1;
            if self.state_timer == 0 && self.state != PetState::Sleeping {
                self.state = PetState::Idle;
            }
        }

        self.decay_ticks += 1;
        if self.decay_ticks >= 40 {
            self.decay_ticks = 0;
            if self.state == PetState::Sleeping {
                self.energy = (self.energy + 1).min(4);
            } else {
                self.hunger = self.hunger.saturating_sub(1);
                self.happiness = self.happiness.saturating_sub(1);
            }
        }
    }
}