#!/usr/bin/env bash
# Cloud Agent environment bootstrap for DB Pro.
#
# DB Pro is a native Rust desktop app (eframe/egui). The base image already
# ships the Rust toolchain (rustup honours rust-toolchain.toml -> 1.95.0) and a
# SQLite CLI, but the native windowing / GL / Wayland / D-Bus development headers
# that eframe + keyring link against are not present by default. Install those
# system packages, warm the cargo registry, and pre-build the shipped binary so
# a booted agent starts with a ready release build.
#
# Idempotent: apt-get install and cargo build are safe to re-run.
set -euo pipefail

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  SUDO="sudo"
fi

export DEBIAN_FRONTEND=noninteractive

$SUDO apt-get update -qq
$SUDO apt-get install -y --no-install-recommends \
  pkg-config \
  libx11-dev \
  libxcb1-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libwayland-dev \
  libgl1-mesa-dev \
  libegl1-mesa-dev \
  libdbus-1-dev \
  libxi-dev \
  libxcursor-dev \
  libxrandr-dev

# Fetch dependencies against the committed lockfile, then build the shipped
# native binary (the UI quality gate) so the snapshot carries a warm target dir.
cargo fetch --locked
cargo build --release --locked -p db-pro-native

echo "DB Pro cloud environment bootstrap complete."
