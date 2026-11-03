#!/bin/bash

set -euo pipefail

cargo build --target wasm32-unknown-unknown --no-default-features --features web

wb_version="0.2.130"
wb_root="$PWD/tools/wasm-bindgen/$wb_version"
if [ ! -x "$wb_root/bin/wasm-bindgen" ]; then
    cargo install wasm-bindgen-cli --version "$wb_version" --root "$wb_root" --locked
fi

"$wb_root/bin/wasm-bindgen" --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "philosopher_stone" \
    ./target/wasm32-unknown-unknown/debug/philosopher_stone.wasm

rm -r out/assets || echo "no assets folder in out"
cp -r assets out/assets
cp web/index.html out/index.html
