#!/usr/bin/env bash
# source: https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/

set -euo pipefail
IFS=$'\n\t'
RBM_BUILD_ONLY="${RBM_BUILD_ONLY:-}"

[ -z "${RBM_BUILD_ONLY}" ] && ./postgres.sh

pushd frontend
# note: when using SpaRouter this needs to be "trunk build --public-url /"
trunk build
popd

cargo build --release

[ -z "${RBM_BUILD_ONLY}" ] && cargo run --release