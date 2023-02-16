#!/usr/bin/env bash

docker network ls | grep rbm || docker network create rbm

if docker ps -a | grep rbm-postgres; then
  docker start rbm-postgres
else
  docker run -d \
    --name rbm-postgres \
    -e POSTGRES_PASSWORD=password \
    -v rbm-postgres:/var/lib/postgresql/data \
    -p 5432:5432 \
    --network rbm \
    postgres
fi