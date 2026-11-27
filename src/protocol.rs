pub const SYNC_0: u8 = 0xAA;
pub const SYNC_1: u8 = 0x55;
pub const FRAME_LEN: usize = 16;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    CoreTelemetry = 0x01,
    DockIdentify = 0x02,
    DockStimulus = 0x03,
    DockCredits = 0x04,
    ScriptChunk = 0x10,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DockShell {
    None = 0x00,
    Coliseum = 0x01,
    AiAssistant = 0x02,
    Rover = 0x03,
}

impl DockShell {
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x01 => DockShell::Coliseum,
            0x02 => DockShell::AiAssistant,
            0x03 => DockShell::Rover,
            _ => DockShell::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DockShell::None => "",
            DockShell::Coliseum => "ARENA",
            DockShell::AiAssistant => "AI-BASE",
            DockShell::Rover => "ROVER",
        }
    }
}

/// Fixed 12-byte core payload with 16-bit spendable step currency (`credits`)[cite: 6]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CorePayload {
    pub species_id: u8,  // 0x01 = Brain-in-a-Jar Wetware Core[cite: 6]
    pub level: u8,       // Derived from lifetime kinetic steps[cite: 6]
    pub xp_lo: u8,       // Low byte of lifetime steps[cite: 6]
    pub xp_hi: u8,       // High byte of lifetime steps[cite: 6]
    pub fluid: u8,       // 0..=4 nutrient level[cite: 6]
    pub lucidity: u8,    // 0..=4 clarity / engagement[cite: 6]
    pub static_load: u8, // 0..=4 synaptic noise[cite: 6]
    pub mood: u8,        // Face Mood enum discriminant[cite: 6]
    pub bond: u8,        // Interaction / glass-tap affinity[cite: 6]
    pub credits_lo: u8,  // Low byte of spendable step currency[cite: 6]
    pub credits_hi: u8,  // High byte of spendable step currency[cite: 6]
    pub reserved: u8,    // Reserved byte for future flags[cite: 6]
}

impl CorePayload {
    pub fn to_bytes(&self) -> [u8; 12] {
        [
            self.species_id,
            self.level,
            self.xp_lo,
            self.xp_hi,
            self.fluid,
            self.lucidity,
            self.static_load,
            self.mood,
            self.bond,
            self.credits_lo,
            self.credits_hi,
            self.reserved,
        ]
    }
}

pub enum IncomingPacket {
    ShellDocked(DockShell),
    Stimulus { code: u8, intensity: u8 },
    CreditsDelta { spend: bool, amount: u16 },
}

pub struct PacketFramer {
    rx_buf: [u8; FRAME_LEN],
    rx_idx: usize,
}

impl PacketFramer {
    pub const fn new() -> Self {
        Self {
            rx_buf: [0; FRAME_LEN],
            rx_idx: 0,
        }
    }

    pub fn compute_checksum(buf: &[u8]) -> u8 {
        let mut csum = 0u8;
        for &b in &buf[..FRAME_LEN - 1] {
            csum ^= b;
        }
        csum
    }

    pub fn encode_telemetry(payload: &CorePayload) -> [u8; FRAME_LEN] {
        let mut frame = [0u8; FRAME_LEN];
        frame[0] = SYNC_0;
        frame[1] = SYNC_1;
        frame[2] = PacketType::CoreTelemetry as u8;
        frame[3..15].copy_from_slice(&payload.to_bytes());
        frame[15] = Self::compute_checksum(&frame);
        frame
    }

    pub fn push_byte(&mut self, byte: u8) -> Option<IncomingPacket> {
        if self.rx_idx == 0 {
            if byte == SYNC_0 {
                self.rx_buf[0] = byte;
                self.rx_idx = 1;
            }
        } else if self.rx_idx == 1 {
            if byte == SYNC_1 {
                self.rx_buf[1] = byte;
                self.rx_idx = 2;
            } else if byte == SYNC_0 {
                self.rx_buf[0] = SYNC_0;
                self.rx_idx = 1;
            } else {
                self.rx_idx = 0;
            }
        } else {
            self.rx_buf[self.rx_idx] = byte;
            self.rx_idx += 1;

            if self.rx_idx == FRAME_LEN {
                self.rx_idx = 0;
                if Self::compute_checksum(&self.rx_buf) == self.rx_buf[15] {
                    return Self::decode_frame(&self.rx_buf);
                }
            }
        }
        None
    }

    fn decode_frame(frame: &[u8; FRAME_LEN]) -> Option<IncomingPacket> {
        match frame[2] {
            0x02 => Some(IncomingPacket::ShellDocked(DockShell::from_byte(frame[3]))),
            0x03 => Some(IncomingPacket::Stimulus {
                code: frame[3],
                intensity: frame[4],
            }),
            0x04 => {
                let spend = frame[3] != 0;
                let amount = u16::from_le_bytes([frame[4], frame[5]]);
                Some(IncomingPacket::CreditsDelta { spend, amount })
            }
            _ => None,
        }
    }
}