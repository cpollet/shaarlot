#!/bin/bash

docker run -d \
  --name rbm-postgres \
  -e POSTGRES_PASSWORD=password \
  -v rbm-postgres:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres