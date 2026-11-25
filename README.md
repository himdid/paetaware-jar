<img width="508" height="616" alt="it begins" src="https://github.com/user-attachments/assets/c819b44f-f408-46db-b58b-12a546037dd3" />

# PaetaWare ONE

Bare-metal `#![no_std]` Rust firmware for a virtual digital pet handheld, targeting the **Seeed XIAO SAMD21** (`ATSAMD21G18A`, ARM Cortex-M0+) and a **0.96" SSD1306 128×64 I2C OLED** display.

Currently developed and verified in the **Mokxi** circuit simulator prior to breadboard and custom 3D-printed enclosure assembly.

## Hardware & Pinout

| Peripheral | Signal | XIAO Pin | SAMD21 Port | Notes |
| :--- | :--- | :--- | :--- | :--- |
| SSD1306 OLED (128×64) | `SDA` | `D4 / A4` | `PA08` | Software bit-banged I2C (`0x3C`) |
| SSD1306 OLED (128×64) | `SCL` | `D5 / A5` | `PA09` | Software bit-banged I2C |
| Onboard Status LED | `L` | Built-in | `PA17` | Active-low heartbeat indicator |

## Project Structure

```text
├── .cargo/config.toml   # Cortex-M0+ target (thumbv6m-none-eabi) & linker flags
├── memory.x             # Flash/RAM memory layout (0x00000000 for simulator ELF)
└── src/
    ├── main.rs          # Peripheral init & main animation/input loop
    ├── display.rs       # Bit-banged SoftI2c driver, SSD1306 setup & 2x sprite scaler
    └── sprites.rs       # 1-bit monochrome bitmap frames (16×16)
```

## Roadmap:

[x] Bare-metal no_std toolchain & Cortex-M0+ linker setup

[x] Custom SoftI2c driver & SSD1306 framebuffer rendering

[x] 2× nearest-neighbor sprite scaling & multi-frame idle animation

[ ] Tactile button inputs with internal pull-ups & debouncing

[ ] Virtual pet state machine (hunger, happiness, energy, sleep cycles)

[ ] Passive buzzer chiptune/feedback tones

[ ] To be announced...

## License

Licensed under the [MIT License](LICENSE).
