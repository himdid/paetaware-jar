use atsamd_hal::fugit::RateExtU32;
use atsamd_hal::gpio::{AlternateD, Pin, PB08, PB09};
use atsamd_hal::pac::{Gclk, Pm, Sercom4};
use atsamd_hal::sercom::uart::{self, BaudMode, Oversampling};
use embedded_hal::blocking::serial::Write as BlockingWrite;
use embedded_hal::serial::Read;

pub type Uart4Pads = uart::Pads<
    Sercom4,
    Pin<PB09, AlternateD>, // D7: RX[cite: 6]
    Pin<PB08, AlternateD>, // D6: TX[cite: 6]
>;

pub type Uart4Port = uart::Uart<uart::Config<Uart4Pads>, uart::Duplex>;

pub struct XiaoUart {
    uart: Uart4Port,
}

impl XiaoUart {
    pub fn new(
        sercom4: Sercom4,
        pm: &mut Pm,
        gclk: &mut Gclk,
        tx_pin: Pin<PB08, AlternateD>,
        rx_pin: Pin<PB09, AlternateD>,
    ) -> Self {
        // Route default GCLK0 (1 MHz OSC8M) to SERCOM4_CORE (ID 0x18) without touching DFLL48M[cite: 6]
        gclk.clkctrl().write(|w| unsafe {
            w.id().bits(0x18)
                .gen()
                .gclk0()
                .clken()
                .set_bit()
        });
        while gclk.status().read().syncbusy().bit_is_set() {}

        let pads = uart::Pads::default().rx(rx_pin).tx(tx_pin);

        let uart = uart::Config::new(pm, sercom4, pads, 1_000_000u32.Hz())
            .baud(9_600u32.Hz(), BaudMode::Fractional(Oversampling::Bits16))
            .enable();

        Self { uart }
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        self.uart.read().ok()
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        let _ = self.uart.bwrite_all(bytes);
    }
}