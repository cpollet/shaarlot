#!/usr/bin/env bash
# source: https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/

set -euo pipefail
IFS=$'\n\t'

rustup target add wasm32-unknown-unknown
cargo install cargo-watch
cargo install trunk

./postgres.sh

mkdir -p target/debug/wasm

# FIXME `trunk serve` not work with /assets/static/
# starts:
#  - trunk on port 8000
#  - rbm on port 3000
(trap 'kill 0' SIGINT; \
 bash -c 'cd frontend; trunk serve --public-url / --address 0.0.0.0 --proxy-backend=http://localhost:3000/api/' & \
 bash -c 'cargo watch -- ROOT_PATH=target/debug/wasm ASSETS_URL=/assets cargo run')

# start:
#  - rbm on port 3000
#(trap 'kill 0' SIGINT; \
# bash -c 'cd frontend; trunk watch --public-url /assets --dist ../target/debug/wasm' & \
# bash -c 'cargo watch -- ROOT_PATH=target/debug/wasm cargo run')