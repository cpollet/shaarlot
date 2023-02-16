#!/usr/bin/env bash

./postgres.sh

RBM_BUILD_ONLY=1 ./prod.sh

cp target/release/rbm docker/rbm
cp -r dist docker/

pushd docker
docker build -t rbm .
popd

docker run --rm \
  --name rbm \
  -e DATABASE_HOST=rbm-postgres \
  -p 3000:3000 \
  --network rbm \
  rbm