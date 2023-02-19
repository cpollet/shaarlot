#!/usr/bin/env bash

./postgres.sh

RBM_BUILD_ONLY=1 ./prod.sh

rm -r docker/target; mkdir docker/target
cp target/release/rbm docker/target/rbm
cp -r target/release/wasm docker/target/webroot

pushd docker
docker rmi cpollet/rbm
docker build -t cpollet/rbm .
popd

docker run --init --rm \
  --name rbm \
  -e DATABASE_HOST=rbm-postgres \
  -e HTTP_PORT=8001 \
  -p 8001:8001 \
  --network rbm \
  cpollet/rbm