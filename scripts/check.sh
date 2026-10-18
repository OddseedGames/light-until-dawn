#!/bin/sh
set -eu
. "$(dirname -- "$0")/rust-env.sh"
cargo fmt --check
cargo test --all-targets --locked
cargo check --target wasm32-unknown-unknown --locked
