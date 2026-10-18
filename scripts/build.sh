#!/bin/sh
set -eu
. "$(dirname -- "$0")/rust-env.sh"
cargo build --release --target wasm32-unknown-unknown --locked
mkdir -p dist
cp web/index.html web/style.css web/mq_js_bundle.js web/THIRD-PARTY-LICENSES.txt dist/
cp assets/IBMPlexMono-Regular.ttf assets/OFL.txt dist/
cp target/wasm32-unknown-unknown/release/until-dawn.wasm dist/
python3 scripts/package.py
python3 scripts/verify-package.py
printf 'Готово: %s/dist\n' "$task_project_dir"
