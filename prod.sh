#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

./postgres.sh

pushd frontend
# note: when using SpaRouter this needs to be
#   "trunk build --public-url /"
trunk build
popd

cargo run --release