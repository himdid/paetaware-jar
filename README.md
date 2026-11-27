<img width="596" height="567" alt="jartime" src="https://github.com/user-attachments/assets/8211efa1-77a8-48c2-971e-cff7c030cb4b" />


# PaetaWare Jar (`peata-ware-jar`)

Bare-metal `#![no_std]` Rust firmware for **PaetaWare Jar**, a modular, pocket-sized virtual digital pet ("Brain-in-a-Jar" core) featuring a procedural animated face, a step-minted economy, physical stasis switch, and magnetic UART docking for external shells (AI Desk Base, Rover, etc).

Designed to function both as a standalone pocket containment unit and as a plug-in neural core that docks into external hardware shells over UART. 

Currently developed and verified in the **Mokxi** circuit simulator prior to breadboard and custom 3D-printed enclosure assembly.

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



---

## Architecture Overview

The firmware is strictly decoupled into a hardware-agnostic 
**Core Engine** and a board-specific **Hardware Abstraction Layer (HAL)**. 
This keeps game logic portable across microcontrollers (Seeed XIAO SAMD21, RP2040), enables fast host testing, and prepares the core for hot-loadable `.paeta` bytecode scripts over UART.

```text
src/
├── main.rs                 # Entry point: initializes HAL and runs the PetEngine tick loop
├── platform.rs             # Hardware boundary trait (PetPlatform) & RenderScene snapshot
├── protocol.rs             # Zero-HAL 16-byte UART framing, checksums, and packet definitions
├── core/
│   ├── mod.rs              # PetEngine orchestrator (input routing, tick cadence, telemetry)
│   ├── pet.rs              # Pure state machine: vitals, step economy, menus, and jar bubbles
│   └── face.rs             # Procedural eye pose interpolation, blinking, glances, and moods
└── hal_xiao/
    ├── mod.rs              # Concrete XiaoBoard implementing PetPlatform for XIAO SAMD21
    ├── display.rs          # Bit-banged SoftI2c + SSD1306 128x64 procedural renderer
    ├── input.rs            # Debounced button polling & physical stasis slide-switch reader
    └── uart.rs             # SERCOM4 non-blocking UART driver (9600 baud)
