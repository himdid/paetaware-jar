<img width="508" height="616" alt="it begins" src="https://github.com/user-attachments/assets/c819b44f-f408-46db-b58b-12a546037dd3" />

# PaetaWare ONE

Bare-metal `#![no_std]` Rust firmware for a modular **"Brain-in-a-Jar" wetware specimen core**, targeting the **Seeed XIAO SAMD21** (`ATSAMD21G18A`, ARM Cortex-M0+) and a **0.96" SSD1306 128×64 I2C OLED** display.

Designed to function both as a standalone pocket containment unit and as a plug-in neural core that docks into external hardware shells (Raspberry Pi Pico docking station, desktop AI assistant base, autonomous rover, or battle arena) over UART. Currently developed and verified in the **Mokxi** circuit simulator prior to breadboard and custom 3D-printed enclosure assembly.

## Features

* **Procedural Eye & Eyelid Engine (`face.rs`):** Smoothly interpolated `EyePose` primitives (`Calm`, `Happy`, `Angry`, `Dizzy`, `Sleepy`, `Asleep`, `Surprised`, `Lonely`) with autonomous blinking, random glances, rapid glass-tap temper (5 quick pokes), shake-to-dizzy swirl, and 30-second idle loneliness.
* **Clean Containment Viewport (`display.rs`):** Fullscreen expressive eyes with rising nutrient fluid bubbles and an auto-hiding `COIN` / `PING | DOSE | SCAN` HUD that clears automatically when idle or in Stasis.
* **Specimen Vitals & Step Currency (`pet.rs`):** Tracks nutrient **Fluid**, **Lucidity**, **Static Load**, lifetime **Steps** (XP), and spendable **Step Coins** (`+1 COIN` per 10 steps) used for electrolyte flushes (`DOSE`) and docked accessory interactions.
* **Modular Dock UART Protocol (`protocol.rs`):** Fixed 16-byte `0xAA 0x55` XOR-checksummed framing over `SERCOM4` (`9,600` baud) broadcasting live specimen telemetry and receiving shell identification (`ARENA`, `AI-BASE`, `ROVER`), stimulus events, and coin transactions.

## Hardware & Pinout

### Core Components
* **MCU:** Seeed Studio XIAO SAMD21 (`ATSAMD21G18A`, ARM Cortex-M0+)
* **Display:** 0.96" SSD1306 128×64 I2C OLED (`0x3C`)
* **Controls & Sensors:** 4× tactile pushbuttons (`D0`–`D3`), slide switch (`D9`), kinetic step/shake switch (`D10`)
* **Planned Hardware:** Passive piezo buzzer (`D8`), 3-pin magnetic pogo / TRS dock connector (`D6`/`D7`/`GND`), CR2450 coin-cell / LiPo power

### Pin Mapping

| Peripheral | Signal | XIAO Pin | SAMD21 Port | Notes |
| :--- | :--- | :--- | :--- | :--- |
| Navigation Left | `BTN_L` | `D0 / A0` | `PA02` | Internal pull-up, active-low |
| Navigation Right | `BTN_R` | `D1 / A1` | `PA04` | Internal pull-up, active-low |
| Confirm / Glass Tap (`PING`) | `BTN_A` | `D2 / A2` | `PA10` | Internal pull-up, active-low |
| Cancel / Hide HUD | `BTN_B` | `D3 / A3` | `PA11` | Internal pull-up, active-low |
| SSD1306 OLED (128×64) | `SDA` | `D4 / A4` | `PA08` | Software bit-banged I2C (`0x3C`) |
| SSD1306 OLED (128×64) | `SCL` | `D5 / A5` | `PA09` | Software bit-banged I2C |
| Modular Dock Link | `UART_TX` | `D6` | `PB08` | `SERCOM4` TX (`9,600` baud, 8N1) |
| Modular Dock Link | `UART_RX` | `D7` | `PB09` | `SERCOM4` RX (`9,600` baud, 8N1) |
| Cryo-Stasis Switch | `STASIS` | `D9 / A9` | `PA05` | Slide switch; puts specimen into Stasis |
| Kinetic Agitation / Pedometer | `STEP` | `D10 / A10` | `PA06` | Tilt/pedometer pulse input (`+COIN` / jolt) |
| Onboard Status LED | `L` | Built-in | `PA17` | Active-low telemetry heartbeat indicator |

## Project Structure

```text
├── .cargo/config.toml   # Cortex-M0+ target (thumbv6m-none-eabi) & linker flags
├── memory.x             # Flash/RAM memory layout (0x00000000 for simulator ELF)
└── src/
    ├── main.rs          # Peripheral init, SERCOM4 dock polling & ~20 FPS loop
    ├── display.rs       # Bit-banged SoftI2c driver, procedural eyes & auto-hiding HUD
    ├── face.rs          # Parametric EyePose interpolation & autonomous mood engine
    ├── input.rs         # Debounced pull-up button, step & stasis switch polling
    ├── pet.rs           # Brain-in-a-Jar state machine, vitals, bubbles & step currency
    └── protocol.rs      # Fixed 16-byte UART framing & dock shell handshake (SERCOM4)