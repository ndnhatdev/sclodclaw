# REDHORSE FIRMWARE KNOWLEDGE BASE

## OVERVIEW
`firmware/` is the embedded lane for Redhorse.
It spans multiple targets with different toolchains, so treat each target directory as its own build surface rather than assuming one shared firmware workflow.

## STRUCTURE
```text
firmware/
|- esp32/          # Rust ESP-IDF target
|- esp32-ui/       # Rust + Slint UI target
|- nucleo/         # Rust MCU target
|- arduino/        # single-file Arduino sketch
`- uno-q-bridge/   # bridge app with sketch/ and python/ helpers
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| ESP32 firmware | `esp32/`, `esp32/Cargo.toml`, `esp32/SETUP.md` | Rust target with dedicated toolchain setup |
| ESP32 UI firmware | `esp32-ui/`, `esp32-ui/ui/` | Rust target plus embedded Slint assets |
| Nucleo firmware | `nucleo/` | small Rust target crate |
| Arduino sketch | `arduino/arduino.ino` | standalone Arduino flow |
| UNO bridge | `uno-q-bridge/sketch/`, `uno-q-bridge/python/` | mixed sketch + helper tooling |

## CONVENTIONS
- Validate changes with the target-local manifest, README, or setup doc; there is no single firmware-wide command that covers every target here.
- Keep hardware- or board-specific assumptions inside the target directory that owns them.
- Treat `esp32-ui/` as both firmware and UI asset work; changes can span Rust and `ui/` resources together.

## ANTI-PATTERNS
- Do not run root `cargo` commands and assume they verify firmware targets.
- Do not copy configuration, linker, or board assumptions across targets without checking the local setup files.
- Do not treat `arduino/` or `uno-q-bridge/` as Rust crates; their workflows differ from `esp32/` and `nucleo/`.

## COMMANDS
```bash
cargo build --manifest-path firmware/esp32/Cargo.toml
cargo build --manifest-path firmware/esp32-ui/Cargo.toml
cargo build --manifest-path firmware/nucleo/Cargo.toml
```

## NOTES
- Use the nearest target README or setup file before editing build flags or flashing flow.
- Repo-root `CLAUDE.md` still governs risk and validation discipline for high-impact changes.
