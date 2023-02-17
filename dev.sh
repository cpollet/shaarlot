#!/usr/bin/env bash
# source: https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/

set -euo pipefail
IFS=$'\n\t'

./postgres.sh

(trap 'kill 0' SIGINT; \
 bash -c 'cd frontend; trunk serve --address 0.0.0.0 --proxy-backend=http://localhost:3000/api/' & \
 bash -c 'cargo watch -- cargo run')