#!/usr/bin/env bash

cargo clean

pushd frontend
trunk clean
popd

rm -rf docker/dist
rm -rf docker/rbm