#!/usr/bin/env bash
# Builds the Rust core (arm64) that EVKeyKit links: target/release/libevkey_core.a.
# Universal (arm64 + x86_64) packaging is handled by the packaging scripts.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
cargo build -p evkey-core --release
