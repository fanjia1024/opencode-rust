#!/usr/bin/env bash
# 发布构建，产出 target/release/opencode。使用 Cargo 默认配置。

set -e
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.."

command -v cargo &>/dev/null || { echo "Error: cargo not found. Install Rust from https://rustup.rs"; exit 1; }
[[ -f Cargo.toml && -d opencode-cli ]] || { echo "Error: Run from opencode-rust project root."; exit 1; }

cargo build --workspace --release
