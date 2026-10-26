#!/bin/bash
# Cross-Build für Windows von Linux aus (mingw-w64 nötig). Mit rustup einfacher:
#   rustup target add x86_64-pc-windows-gnu && cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
