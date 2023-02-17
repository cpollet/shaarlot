#!/usr/bin/env bash
# source: https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/

set -euo pipefail
IFS=$'\n\t'
RBM_BUILD_ONLY="${RBM_BUILD_ONLY:-}"

[ -z "${RBM_BUILD_ONLY}" ] && ./postgres.sh

cargo build --release

pushd frontend
trunk build --public-url /assets --dist ../target/release/wasm --release
popd

[ -z "${RBM_BUILD_ONLY}" ] && HTTP_PORT=8000 ROOT_PATH=target/release/wasm cargo run --release