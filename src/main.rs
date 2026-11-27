#![no_std]
#![no_main]

mod core;
mod hal_xiao;
mod platform;
mod protocol;

use atsamd_hal::pac::Peripherals;
use cortex_m_rt::entry;
use panic_halt as _;

use crate::core::PetEngine;
use crate::hal_xiao::XiaoBoard;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut board = XiaoBoard::init(peripherals);
    let mut engine = PetEngine::new();

    engine.boot(&mut board);

    loop {
        engine.update(&mut board);
    }
}