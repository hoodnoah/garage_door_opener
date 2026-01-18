# ESP32 Rust
# Set your target chip before running any commands
# Options: esp32, esp32s2, esp32s3, esp32c2, esp32c3, esp32c6, esp32h2

TARGET := "esp32c3"

# Libs
# Set your development target for libraries to be tested as part of this project.
# Options: aarch64-apple-darwin

DEV_TARGET := "aarch64-apple-darwin"
LIB_NAME := ""

default:
    @just --list

# One-time setup
setup:
    cargo install espup
    cargo install cargo-espflash espflash
    espup install --targets "{{ TARGET }}"
    @echo "Restart shell: exit, then nix develop"

new:
    cargo generate esp-rs/esp-idf-template --name main
    cp -r main/.cargo .cargo

build:
    cargo build --release

flash:
    cargo espflash flash --release --monitor

monitor:
    cargo espflash monitor

clean:
    cargo clean

test-lib:
    cargo test -p "{{ LIB_NAME }}" --target "{{ DEV_TARGET }}"
